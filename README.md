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

**Phase 1 — Bootstrapping** (in progress)

The hypervisor can boot, initialize the hardware environment, parse platform topology from the Device Tree Blob (DTB), and set up early UART output. There is no VM lifecycle yet — Stage-2 MMU, interrupt virtualization, and vCPU management have not been started.

See [`docs/roadmap.md`](docs/roadmap.md) for the full plan.

---

## Build, Emulation & Debug

All scripts accept `--target <arch>` to select the target architecture and `--help` to show usage. The `TARGET_TRIPLE` env var is also supported and is overridden by `--target`. Default target is `aarch64`.

```bash
# Check that all required tools are present
bash scripts/doctor.sh [--target aarch64]

# Run formatter + clippy for the target
bash scripts/check.sh [--target aarch64]

# Build the hypervisor image
bash scripts/build.sh [--target aarch64]

# QEMU bring-up — serial output on stdout
bash scripts/qemu-run.sh [--target aarch64]

# QEMU with trace flags; output written to qemu.log
bash scripts/qemu-trace.sh [--target aarch64]

# Start QEMU paused with a GDB stub on :1234
bash scripts/qemu-gdb.sh [--target aarch64]

# Connect GDB to a paused QEMU instance (run in a second terminal)
bash scripts/gdb.sh [--target aarch64]
```

The QEMU binary, machine, and CPU are derived automatically from the target. They can be overridden via `QEMU_BIN`, `QEMU_MACHINE`, and `QEMU_CPU`. The QEMU scripts are bring-up aids scoped to the `virt` machine; they are not a statement about the supported deployment targets.

---

## Architecture

See [`docs/architecture.md`](docs/architecture.md).

---

## Contributing

See [`CONTRIBUTING.md`](CONTRIBUTING.md).

---

## License

MIT
