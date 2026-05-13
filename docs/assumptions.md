# Design Assumptions

These are the invariants hemitheos relies on. Violating any of them results in undefined behavior or an early halt.

---

## Hardware

- ARMv8-A AArch64 CPU
- Execution starts in Non-secure EL2; the boot entry halts if the CPU is in any other EL
- EL2 is present and not restricted by firmware (no `HCR_EL2.E2H` tricks from prior stages)

## Firmware / Boot Protocol

- The bootloader follows the ARM64 Linux Image boot protocol: loads the binary at the address encoded in the Image header and passes a firmware argument in `x0`
- The firmware argument is accessible before the stack or BSS are initialized (saved in `x20` at the first instruction)
- At least one platform information source is available: a valid DTB, ACPI tables, or firmware-provided data. The hypervisor halts if none can be identified.
- If the boot source is a DTB, it contains at minimum:
  - A `/memory` node with at least one valid RAM region
  - A `/cpus` node with at least one CPU
  - A UART node reachable via `/chosen/stdout-path`, `/aliases/serial0`, or a compatible-string scan *(optional — boot continues without console if absent)*

## Memory

- The image is loaded at `0x40080000` on QEMU virt (encoded via `text_offset` in the Image header)
- At least 64 KiB of RAM is available above the end of `.bss` for the initial stack
- No external agent modifies the hypervisor's memory after boot

## Concurrency

- Until secondary CPUs are explicitly woken (Phase 3), all execution is single-threaded on CPU 0
- Global initialization primitives (`Once<T>`) enforce single-writer semantics; this invariant is maintained by the SMP bringup sequence when secondaries are enabled
