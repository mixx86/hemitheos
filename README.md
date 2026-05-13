# hemitheos

A minimal, auditable Type-1 bare-metal hypervisor written in Rust.

> **Status: early development — bootstrapping stage.**
>
> **Architecture:** AArch64 is the primary and only supported target for now. The build system and scripts are structured to accommodate other architectures (e.g. RISC-V) in the future, once the project is more mature. Contributions or proposals for new architecture ports are welcome, but will not be prioritized until the AArch64 base is stable.

---

## Overview

hemitheos aims to be a secure, minimal hypervisor that scales from constrained hardware to larger deployments. The core stays lean — functionality is added through extensions installed from a privileged domain (a dom0-equivalent), so the base never pays the cost of features it does not use. The primary extension planned is network-level orchestration: forming clusters of hypervisor nodes, managing VMs across them, and controlling access — roughly what Proxmox offers, but built on this codebase. Other extensions (backup management, scheduling, permissions) follow the same model.

The project is ARM-first. AArch64 is the primary target and will remain so until the base is stable. Other architectures may follow.

---

## Current Status

**Phase 1 — Bootstrapping** ✓ complete

**Phase 2 — Hypervisor Memory Management** (in progress)

The hypervisor boots on QEMU and real AArch64 hardware, initializes the hardware environment, discovers platform topology (boot source: DTB, ACPI, or firmware), sets up early UART output with dynamic baud rate, and parks secondary cores. There is no VM lifecycle yet — Stage-2 MMU, interrupt virtualization, and vCPU management have not been started.

See [`docs/roadmap.md`](docs/roadmap.md) for the full plan.

---

## Repository Layout

```
src/
  main.rs             Binary entry point (gated by --features bin-target)
  lib.rs              Crate root
  arch/
    mod.rs            Arch dispatch stubs (host compilation)
    arm64/            AArch64-specific code
      head.S          Boot entry point (ARM64 Image protocol)
      entry.S         EL2 vector table and exception entry stubs
      hemitheos.lds.S Linker script
      cpu.rs          CPU topology discovery (MPIDR, probe chain)
      traps.rs        EL2 exception handlers
      bringup.rs      Hardware bringup orchestration (Phase 2+)
      gic.rs          GIC driver (Phase 3+)
      timer.rs        Timer driver (Phase 3+)
  boot.rs             BootSource enum and probe/detect dispatch
  common/
    platform.rs       Platform init — memory, CPUs, console
    console.rs        ConsoleDriver trait, discovery, global registry
    log.rs            Ring-buffer logging with per-CPU buffers
    sync.rs           Once<T> and SpinLock primitives
  drivers/
    uart/pl011.rs     PL011 UART driver

build.rs              Assembles src/arch/arm64/ ASM via cc crate
scripts/              Build, test, QEMU, and GDB helper scripts
tests/                Integration tests (host-runnable)
docs/                 Architecture, assumptions, and roadmap
```

---

## Build, Emulation & Debug

All scripts accept `--target <arch>` and `--help`. Default target is `aarch64`.

```bash
# Check that all required tools are present
bash scripts/doctor.sh

# Build the hypervisor image
bash scripts/build.sh

# Run host integration tests
bash scripts/test.sh

# QEMU bring-up — serial output on stdout
bash scripts/qemu-run.sh

# QEMU with trace flags; output written to qemu.log
bash scripts/qemu-trace.sh

# Start QEMU paused with a GDB stub on :1234
bash scripts/qemu-gdb.sh

# Connect GDB to a paused QEMU instance
bash scripts/gdb.sh

# Run formatter + clippy
bash scripts/check.sh
```

The QEMU binary, machine, and CPU are derived from the target architecture and can be overridden via `QEMU_BIN`, `QEMU_MACHINE`, and `QEMU_CPU`.

---

## Testing

Integration tests run natively on the host (no cross-compilation needed):

```bash
bash scripts/test.sh
```

The binary target (`src/main.rs`) requires the `bin-target` feature and the AArch64 toolchain; it is excluded from host test compilation automatically. End-to-end validation runs via QEMU (`scripts/qemu-run.sh`).

---

## Architecture

See [`docs/architecture.md`](docs/architecture.md).

---

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md).

---

## License

MIT
