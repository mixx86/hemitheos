# Roadmap

The project is organized in phases that reflect technical dependency order, not calendar priority. Early phases build the foundation the later ones require. The long-term goal — a privileged domain, an extension model, and network-level orchestration — is described in the README; this document tracks how to get there.

Items within each phase are ordered by dependency where relevant.

---

## Phase 1 — Bootstrapping

Get the hypervisor to boot on the target hardware, establish a working environment, and produce observable output.

### Done

- [x] ARM64 Image header in `boot.s` (Linux ARM64 boot protocol; firmware address passed in `x0`)
- [x] Multi-CPU primary election via `ldaxr`/`stlxr` atomic flag; secondary cores parked in `wfe` loop
- [x] EL check at entry; halt if not EL2
- [x] EL2 environment setup: TLB/icache invalidation, FP trap disable, `HCR_EL2` configuration, timer access enable
- [x] Stack setup (64 KiB), BSS clear
- [x] EL2 vector table loaded into `VBAR_EL2`
- [x] EL2 synchronous exception handler (reads `ESR_EL2`, panics)
- [x] Platform info discovery: RAM regions, CPU count, GICv3 bases, console (probe chain: DTB → ACPI → firmware)
- [x] `ConsoleDriver` trait + global driver registry via `sync::Once`
- [x] Console discovery from platform info (`/chosen`, `/aliases`, compatible scan)
- [x] PL011 driver: hardware init, baud rate computed from clock frequency read from platform info
- [x] PL011 `puts`: spin on `UARTFR.TXFF`, write to `UARTDR`, drain on `UARTFR.BUSY` before return
- [x] `console::init` wired in `main` from `PlatformInfo`
- [x] Driver globals migrated from `static mut` to `sync::Once`
- [x] CI pipeline: build, host test suite, clippy (cross-target), fmt check
- [x] Integration tests for `sync::Once` (host-runnable)
- [x] `log.rs`: ring buffer + fmt::Write; console::puts gated by log level; print!/println! macros
- [x] Exception vector table: save/restore register frame, structured dispatch to Rust handlers

---

## Phase 2 — Hypervisor Memory Management *(current)*

The hypervisor currently runs with the EL2 MMU disabled — no caching, no memory protection, no virtual addressing. This phase enables the minimum viable memory management before tackling guest isolation.

- [ ] EL2 Stage-1 page tables (`TTBR0_EL2`): map hypervisor code, data, per-CPU stacks, and MMIO regions
- [ ] Enable EL2 MMU (`SCTLR_EL2.M`) with correct memory attributes and permissions
- [ ] Enable instruction and data caches
- [ ] Physical frame allocator over the RAM range discovered at boot
- [ ] Hypervisor heap for dynamic allocations (slab or buddy, or a well-tested no_std crate)
- [ ] `Mmio<T>` type: safe MMIO register abstraction with correct volatile semantics and memory barriers
- [ ] PIE (Position Independent Executable): relocator stub or full PIC so the hypervisor can run at any load address without relinking per board

---

## Phase 3 — SMP

Wake and manage all CPUs. Secondary cores are currently parked indefinitely; this phase brings them into the hypervisor's control.

### Done

- [x] CPU topology discovery: MPIDR-based probe chain (platform info → ACPI → firmware → MPIDR fallback)
- [x] Logical CPU index mapping: `current_cpu_index()` via MPIDR lookup table
- [x] Synchronization primitives: `SpinLock`, `Once`

### Pending

- [ ] PSCI `CPU_ON` calls to wake secondary cores (replaces the current spin-table approach)
- [ ] Per-CPU data structures: dedicated stack, exception stack, per-CPU state block
- [ ] Per-CPU GIC redistributor initialization
- [ ] IPI infrastructure via SGIs (Software Generated Interrupts): send, receive, dispatch
- [ ] CPU topology validation against platform info (`/cpus` node, core/cluster layout)

---

## Phase 4 — Hardware Abstractions

Initialize and manage the hardware the hypervisor depends on before guests can run.

- [ ] GICv3 initialization: distributor (`GICD_CTLR`) and redistributors
- [ ] GICv3 interrupt routing to EL2
- [ ] Hypervisor timer: `CNTHP_CTL_EL2` / `CNTHP_TVAL_EL2` setup for a periodic tick
- [ ] Additional UART drivers: NS16550, DW APB

> GICv3 is the initial target. The driver layer will be abstracted to support GICv2 (older boards) and GICv4 (direct interrupt injection for nested virtualization) without reworking the upper layers.

---

## Phase 5 — Stage-2 MMU

Establish the memory isolation layer that makes virtualization meaningful. Without this, guests share the same physical address space.

- [ ] `VTCR_EL2` configuration (IPA size, translation granule, walk levels)
- [ ] Stage-2 page table types: `S2Table`, `S2Block`, `S2Page`
- [ ] Map / unmap / protect APIs
- [ ] Enable `HCR_EL2.VM`

---

## Phase 6 — VM Lifecycle

The minimum needed to create, run, and destroy a guest.

- [ ] `Vm` struct: VMID, Stage-2 root table, vCPU list, device list
- [ ] `Vcpu` struct: general-purpose register save area, system register state
- [ ] VMID allocation
- [ ] `vcpu_run`: enter guest via `eret` into EL1
- [ ] EL1 trap handling: `HVC`, `SMC`, memory aborts, system register traps
- [ ] vCPU affinity and CPU pinning
- [ ] Multi-core guests: SMP guest bringup via virtualized PSCI

---

## Phase 7 — Interrupt Virtualization

Allow guests to receive and manage interrupts without hypervisor intervention on every delivery.

- [ ] Virtual GICv3 (vGICv3): `ICC`/`ICH` system register emulation
- [ ] Per-CPU vGIC redistributor state
- [ ] List register management for direct interrupt injection
- [ ] IPI virtualization between vCPUs
- [ ] Timer virtualization: `CNTVOFF_EL2`, virtual timer injection into guests

---

## Phase 8 — Device Emulation

Provide guests with virtual devices backed by hypervisor-controlled resources.

- [ ] Virtio-MMIO transport layer
- [ ] Virtio-console (backed by host UART)
- [ ] Virtio-blk (backed by an in-memory ramdisk)

---

## Phase 9 — Real Hardware

Validate everything built so far on physical hardware. QEMU brings up early; real hardware surfaces the rest.

- [ ] Physical board bring-up and validation
- [ ] Device assignment policy (config-based or platform-info overlay)
- [ ] SMMU: Stage-2 translation for DMA-capable devices

---

## Phase 10 — MMU Optimizations

Improvements to the memory management layer once the basics are proven on real hardware.

- [ ] ASID (Address Space ID) allocation and TLB tagging to avoid full TLB flushes on context switch
- [ ] Huge page support (2 MiB / 1 GiB blocks) for hypervisor and guest mappings
- [ ] Lazy allocation: map-on-demand for guest memory regions
- [ ] TLB shootdown protocol for SMP

---

## Phase 11 — Privileged Domain

Introduce a privileged guest (dom0-equivalent) as the control plane for the hypervisor. The goal is to move management responsibility out of the hypervisor core and into an isolated domain, keeping the core lean.

- [ ] Boot a privileged domain as the first guest automatically
- [ ] Hypercall interface for VM create / destroy / configure from the privileged domain
- [ ] Resource assignment model: memory regions, interrupt lines, device nodes
- [ ] Secure communication channel between privileged domain and hypervisor
- [ ] Minimal init environment for the privileged domain

---

## Phase 12 — Extension System

Allow functionality to be added to the privileged domain via extensions installed at runtime, without touching the hypervisor core.

- [ ] Extension isolation model (each extension runs in a controlled context)
- [ ] Extension API / ABI definition
- [ ] Package manager integration in the privileged domain
- [ ] First-party extensions: logging, monitoring, access policy enforcement

---

## Phase 13 — Network Orchestration

The primary extension: allow multiple machines running hemitheos to form a cluster and be managed as a unit.

- [ ] Node discovery and cluster formation protocol
- [ ] Distributed VM placement
- [ ] Cross-node VM migration
- [ ] Cluster-wide access control and policy
- [ ] Management interface (API and/or CLI)

---

## Known Technical Debt

| Item | Location | Notes |
|---|---|---|
| — | — | No known debt at this time |
