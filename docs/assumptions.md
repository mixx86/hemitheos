# Design Assumptions

These are the invariants hemitheos relies on. Violating any of them results in undefined behavior or an early halt.

---

## Hardware

- ARMv8-A AArch64 CPU
- Execution starts in Non-secure EL2 (hypervisor mode); the boot entry halts if the CPU is in any other EL
- EL2 is present and not restricted by firmware (e.g. no `HCR_EL2.E2H` tricks from prior stages)
- A GICv3 interrupt controller is present and discoverable via DTB

## Firmware / Boot Protocol

- The bootloader follows the ARM64 Linux Image boot protocol: loads the binary at the address encoded in the Image header and passes the physical address of a valid DTB in `x0`
- The DTB is accessible at the address in `x0` before the stack or BSS are initialized (the address is saved in `x20` at the very first instruction)
- The DTB contains at minimum:
  - A `/memory` node with at least one valid RAM region
  - A `/cpus` node with at least one CPU
  - An `arm,gic-v3` compatible node with GICD and GICR register regions
  - A UART node reachable via `/chosen/stdout-path`, `/aliases/serial0`, or a compatible-string scan

## Memory

- The image is loaded at `0x40080000` (encoded in the Image header `text_offset` field)
- At least 64 KiB of RAM is available above the end of `.bss` for the initial stack
- No external agent modifies the hypervisor's memory after boot

## Concurrency

- Until secondary CPUs are explicitly woken, all execution is single-threaded on CPU 0
- The UART and driver globals (`static mut`) are written only during this single-threaded init phase; this invariant must be maintained if SMP support is added
