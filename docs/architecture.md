# Architecture

## Target Platform

| Property | Value |
|---|---|
| ISA | ARMv8-A AArch64 *(primary; see note below)* |
| Privilege level | EL2 (Non-secure hypervisor) |
| Boot protocol | ARM64 Image (Linux-compatible) |
| Interrupt controller | GICv3 *(Phase 3; not yet initialized)* |

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

Defined in `src/arch/arm64/hemitheos.lds.S`. The load address is set per-target:

```
<load_base>  .text.boot      ARM64 Image header + boot entry (head.S)
             .exception      EL2 vector table (2 KiB aligned, entry.S)
             .text           Rust kernel code
             .rodata         Read-only data
             .data           Initialized data
             .bss            Zero-initialized statics
             + 64 KiB        Stack (grows down from stack_top)
```

The load address in the linker script must match where the bootloader places the image. For the QEMU `virt` target, RAM starts at `0x40000000` and the ARM64 Image header specifies a `text_offset` of `0x80000`, giving `0x40080000`. On real hardware this will differ.

---

## Boot Flow

```
Bootloader / firmware
  └─ loads hemitheos.bin at RAM_BASE + text_offset (ARM64 Image protocol)
       └─ _img_start  (src/arch/arm64/head.S)
            ├─ branches to _start (skipping Image header)
            ├─ primary CPU election via ldaxr/stlxr on _boot_cpu_online
            │    └─ secondary CPUs → secondary_halt (wfe loop)
            ├─ saves boot_arg (x0) in x20 (callee-saved)
            ├─ reads CurrentEL — halts if not EL2
            ├─ sets SPSel=1 (dedicated EL2 stack pointer)
            ├─ TLB/icache invalidation
            ├─ disables FP/SIMD/SVE/SME trapping (cptr_el2 = 0)
            ├─ configures HCR_EL2: EL1=AArch64, routes SError/IRQ/FIQ to EL2
            ├─ enables EL1 access to physical timer registers (cnthctl_el2)
            ├─ sets up 64 KiB stack
            ├─ loads VBAR_EL2 with vector_table_el2
            ├─ clears .bss
            └─ bl main  (src/main.rs)
                 ├─ platform::init(boot_arg)
                 │    ├─ try DTB: validate magic, parse memory/CPUs/console
                 │    ├─ try ACPI  (TODO)
                 │    └─ try firmware  (TODO)
                 ├─ console::init(platform.console)
                 └─ loop { wfe }
```

---

## Boot Source Abstraction (`src/boot.rs`)

The `BootSource` enum decouples discovery logic from the underlying data format:

```rust
pub enum BootSource<'a> {
    Fdt(&'a fdt::Fdt<'a>),
    Acpi,     // TODO
    Firmware, // TODO
}
```

`boot::probe(source, from_dtb, from_acpi, from_firmware)` dispatches to the appropriate closure based on the variant. `boot::detect` (future) will identify the format from a raw `boot_arg` address and create the `BootSource`.

---

## Platform Module (`src/common/platform.rs`)

`platform::init(boot_arg)` tries each boot source in order and populates a global `PlatformInfo`:

```rust
pub struct PlatformInfo {
    pub console: Option<ConsoleInfo>,
    pub ram_base: u64,
    pub ram_size: u64,
    pub num_cpus: usize,
}
```

CPU topology is discovered via `arch::probe_cpus` which tries: DTB → ACPI → firmware → MPIDR fallback → single-CPU fallback.

---

## Console Subsystem (`src/common/console.rs`)

```rust
pub trait ConsoleDriver: Sync {
    fn puts(&self, s: &str);
}
```

A global `static CONSOLE: Once<&'static dyn ConsoleDriver>` holds the active driver after `console::init` is called. `console::puts(s)` is a no-op if called before init.

### Console discovery

`find_console(source: &BootSource)` walks the boot source in priority order:

1. `/chosen` → `stdout-path` / `stdout` / `linux,stdout-path`
2. `/aliases` → `serial0` / `uart0` / `console`
3. Full compatible-string scan

Recognized compatible strings:

| Compatible | Driver |
|---|---|
| `arm,pl011`, `broadcom,pl011` | PL011 |
| `ns16550a`, `ns16550` | NS16550 *(not yet implemented)* |
| `snps,dw-apb-uart` | DW APB *(not yet implemented)* |
| `qcom,msm-uartdm` | MSM *(not yet implemented)* |

### PL011 driver (`src/drivers/uart/pl011.rs`)

Initialization follows the PL011 TRM: disable → set baud divisors → configure 8N1 + FIFO → enable TX. Baud rate is read from the `current-speed` DTB property; defaults to 115200 if absent.

---

## Exception Handling (`src/arch/arm64/`)

The EL2 vector table (`entry.S`) has 16 entries (4 groups × 4 vectors). Each active vector branches to an entry stub that saves the full register frame (ExceptionFrame, 272 bytes), calls the Rust handler, restores the frame, and issues `eret`.

Active handlers (`traps.rs`):

| Exception | Handler |
|---|---|
| EL2 synchronous | Reads `ESR_EL2` + `FAR_EL2`, panics |
| EL2 IRQ | Logs spurious warning (GIC not yet initialized) |
| EL2 FIQ | Logs spurious warning |
| EL2 SError | Reads `ESR_EL2`, panics |

All other vector entries halt with `b .`.
