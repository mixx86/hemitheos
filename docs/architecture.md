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

Defined in `asm/aarch64/linker.ld`. The load address is set per-target:

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
       └─ _img_start (boot.s)
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
            └─ bl main (Rust)
                 ├─ parse_dtb(x0) → PlatformInfo
                 │    ├─ validate DTB magic (0xD00DFEED)
                 │    ├─ memory regions (ram_base, ram_size)
                 │    ├─ CPU count
                 │    ├─ GICv3 base addresses (AArch64 only)
                 │    └─ UART discovery (find_uart)
                 ├─ uart::init(platform.uart)
                 └─ loop {}
```

---

## UART Subsystem

### Driver trait

`src/uart.rs` defines `UartDriver`:

```rust
pub trait UartDriver: Sync {
    fn puts(&self, s: &str);
}
```

A single global `static mut UART: Option<&'static dyn UartDriver>` holds the active driver after `uart::init` is called. The global is written once during single-threaded init; reads after that are safe.

`uart::puts(s)` is the public API consumed by the rest of the kernel.

### PL011 driver (`src/drivers/uart/pl011.rs`)

Current state — partial implementation:

1. Disable UART (`UARTCR`: clear UARTEN bit)
2. Compute and write baud rate divisors (`UARTIBRD`, `UARTFBRD`) from the clock frequency read from the DTB

Not yet implemented: line control register setup, FIFO enable, TX/RX enable, interrupt masking, and the actual `puts` transmit loop.

The driver instance lives in `static mut DRIVER: Option<Pl011>` inside the module. `pl011::init(base, clock_hz)` initializes it once and returns `&'static dyn UartDriver`.

### UART discovery

`uart::find_uart(fdt)` walks the DTB in priority order:

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

---

## DTB Parsing

`src/dtb.rs` uses the `fdt` crate (no_std) to extract `PlatformInfo` at boot.

Fields present on all architectures:

| Field | Source |
|---|---|
| `uart` | `uart::find_uart` |
| `ram_base`, `ram_size` | `/memory` regions |
| `num_cpus` | `/cpus` node count |

Architecture-conditional fields:

| Field | Arch | Source |
|---|---|---|
| `gicd_base` | AArch64 | First `reg` entry of `arm,gic-v3` node |
| `gicr_base`, `gicr_size` | AArch64 | Second `reg` entry of `arm,gic-v3` node |
| `plic_base` | RISC-V | `reg` entry of `riscv,plic0` / `sifive,plic-1.0.0` node |

---

## Exception Handling

The EL2 vector table (`asm/aarch64/vectors.s`) has 16 entries (4 groups × 4 vectors). Currently:

- **EL2 synchronous from current EL (SPx)** → calls `el2_sync_handler` (Rust)
- All other entries → `b .` (infinite loop / halt)

`el2_sync_handler` reads `ESR_EL2` and calls `panic!`. The panic handler writes `"PANIC!\n"` via `uart::puts` and then parks the CPU in a `wfe` loop.

---

## Interrupt Controller

GICv3 base addresses are extracted from the DTB during `parse_dtb`. No initialization has been implemented yet. See [`roadmap.md`](roadmap.md).
