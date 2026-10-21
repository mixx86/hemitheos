# Roadmap

Rough dependency order — not a schedule or checklist.

| Phase | Focus |
| --- | --- |
| 1 | Boot, EL2 setup, console, traps — **done** |
| 2 | Hypervisor memory management — **in progress** |
| 3+ | SMP, GIC/timers, Stage-2, VMs, devices, hardware, dom0/extensions |

Long-term idea: a small hypervisor core with management and features in a privileged domain and extensions (see project README one-liner). Details belong in code and commits, not here.
