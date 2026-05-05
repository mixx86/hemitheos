# Architecture

## Target Platform

| Property | Value |
|---|---|
| ISA | ARMv8-A AArch64 *(primary; see note below)* |
| Privilege level | EL2 (Non-secure hypervisor) |
| Boot protocol | ARM64 Image (Linux-compatible) |
| Interrupt controller | GICv3 |

The build system is structured to support additional architectures in the future. AArch64 is the only implemented target for now.

### QEMU Bring-Up Defaults

These are the defaults used by the scripts in this repository for reproducible bring-up. They are not constraints of the hypervisor itself.

| Parameter | Default |
|---|---|
| Machine | `virt,virtualization=on,gic-version=3` |
| CPU | `cortex-a72` |
| RAM | `512M` |

All parameters can be overridden via environment variables — see `scripts/qemu-common.sh`.

---

## Memory Layout

Defined in `hv/asm/aarch64/linker.ld`. The load address is set per-target:

```
<load_base>  .text.boot      ARM64 Image header + boot entry
             .exception      EL2 vector table (2 KiB aligned)
             .text           Rust kernel code
             .rodata         Read-only data
             .data           Initialized data
             .bss            Zero-initialized statics
             + 64 KiB        Stack (grows down from stack_top)
```

The load address in the linker script must match where the bootloader or firmware places the image. For the QEMU `virt` bring-up target, RAM starts at `0x40000000` and the ARM64 Image header specifies a `text_offset` of `0x80000`, giving a load address of `0x40080000`. On real hardware this will differ.

The ARM64 Image header at offset 0 signals to any compliant bootloader (not just QEMU) that the DTB address is passed in `x0` at entry, following the Linux ARM64 boot protocol.

---

## Boot Flow

```
Bootloader / firmware
  └─ loads hemitheos.bin at RAM_BASE + text_offset (ARM64 Image protocol)
       └─ _img_start (hv/asm/aarch64/boot.s)
            ├─ branches to _start (skipping Image header)
            ├─ primary CPU election via ldaxr/stlxr on _boot_cpu_online
            │    └─ secondary CPUs → secondary_halt (wfe loop)
            ├─ saves DTB address in x20 (callee-saved)
            ├─ reads CurrentEL — halts if not EL2
            ├─ sets SPSel=1 (dedicated EL2 stack pointer)
            ├─ TLB invalidation (tlbi alle2) + icache invalidation (ic iallu)
            ├─ disables FP/SIMD/SVE/SME trapping (cptr_el2 = 0)
            ├─ configures HCR_EL2: EL1 runs AArch64 (RW), routes SError/IRQ/FIQ to EL2
            ├─ enables EL1 access to physical timer registers (cnthctl_el2)
            ├─ sets up 64 KiB stack
            ├─ loads VBAR_EL2 with vector_table_el2
            ├─ clears .bss
            └─ bl main (hv/src/main.rs)
                 ├─ platform::parse_dtb(x0) → PlatformInfo
                 │    ├─ validate DTB magic (0xD00DFEED)
                 │    ├─ memory regions (ram_base, ram_size)
                 │    ├─ CPU count
                 │    ├─ GICv3 base addresses
                 │    └─ console::find_console → ConsoleInfo
                 ├─ console::init(platform.console)
                 └─ loop { wfe }
```

---

## Console Subsystem

### Driver trait (`lib/src/console.rs`)

```rust
pub trait ConsoleDriver: Sync {
    fn puts(&self, s: &str);
}
```

A single global `static CONSOLE: Once<&'static dyn ConsoleDriver>` holds the active driver after `console::init` is called. `Once<T>` (in `lib/src/sync.rs`) guarantees safe one-time initialization without heap allocation or locks.

`console::puts(s)` is the public API consumed by the rest of the kernel. It is a no-op if called before `init`.

### PL011 driver (`lib/src/drivers/uart/pl011.rs`)

Initialization sequence follows the PL011 TRM:

1. Disable UART (`UARTCR`: clear all bits)
2. Compute and write baud rate divisors (`UARTIBRD`, `UARTFBRD`) from the clock frequency read from the DTB
3. Configure 8N1 line control + FIFO enable (`UARTLCR_H`)
4. Enable UART and TX (`UARTCR`: set `UARTEN | TXE`)

Transmit loop in `puts`: polls `UARTFR.TXFF` before each byte (prevents FIFO overflow), then polls `UARTFR.BUSY` after the last byte (ensures drain before returning).

The driver instance lives in `static DRIVER: Once<Pl011>`. `pl011::init(base, clock_hz)` initializes it once and returns `&'static dyn ConsoleDriver`.

### Console discovery (`lib/src/console.rs`)

`console::find_console(fdt)` walks the DTB in priority order:

1. `/chosen` → `stdout-path` / `stdout` / `linux,stdout-path`
2. `/aliases` → `serial0` / `uart0` / `console`
3. Full compatible-string scan as last resort

Recognized compatible strings:

| Compatible | Driver |
|---|---|
| `arm,pl011`, `broadcom,pl011` | PL011 |
| `ns16550a`, `ns16550` | NS16550 *(not yet implemented)* |
| `snps,dw-apb-uart` | DW APB *(not yet implemented)* |
| `qcom,msm-uartdm` | MSM *(not yet implemented)* |

Clock frequency is read from the UART node's `clock-frequency` property. If absent, the crate follows the first `clocks` phandle and reads `clock-frequency` from the provider node, covering platforms (including QEMU `virt`) that use a fixed-clock provider.

---

## Platform Module (`lib/src/platform.rs`)

`platform::parse_dtb(dtb_addr)` extracts `PlatformInfo` from the DTB at boot using the `fdt` crate (no_std).

```rust
pub struct PlatformInfo {
    pub console: Option<ConsoleInfo>,   // discovered UART
    pub ram_base: u64,
    pub ram_size: u64,
    pub num_cpus: usize,
    pub gicd_base: u64,                 // GICv3 distributor
    pub gicr_base: u64,                 // GICv3 redistributor base
    pub gicr_size: u64,                 // GICv3 redistributor region size
}
```

| Field | Source |
|---|---|
| `console` | `console::find_console` |
| `ram_base`, `ram_size` | `/memory` regions (summed) |
| `num_cpus` | `/cpus` node count |
| `gicd_base`, `gicr_base`, `gicr_size` | First and second `reg` entries of `arm,gic-v3` node |

---

## Exception Handling

The EL2 vector table (`hv/asm/aarch64/vectors.s`) has 16 entries (4 groups × 4 vectors). Currently:

- **EL2 synchronous from current EL (SPx)** → calls `el2_sync_handler` (`hv/src/arch/aarch64/exception.rs`)
- All other entries → `b .` (infinite loop / halt)

`el2_sync_handler` reads `ESR_EL2` and calls `panic!`. The panic handler writes `"PANIC!\n"` via `console::puts` and parks the CPU in a `wfe` loop.

---

## Interrupt Controller

GICv3 base addresses are extracted from the DTB during `parse_dtb`. No initialization has been implemented yet. See [`roadmap.md`](roadmap.md).
