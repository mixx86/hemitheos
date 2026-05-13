// EL2 exception handlers — called from the entry stubs in entry.S.
//
// Handlers that panic (Sync, SError) never return; the restore + eret in the
// stub are unreachable but kept for stack symmetry.
//
// Handlers that do return (IRQ, FIQ) must not issue eret themselves — the stub
// owns that instruction. Putting eret inside a Rust extern "C" function causes
// it to fire before the compiler's epilogue unwinds the call frame.

#[unsafe(no_mangle)]
pub extern "C" fn EL2_Sync_Handler() {
    let esr: u64;
    let far: u64;
    // SAFETY: ESR_EL2 and FAR_EL2 are always readable from EL2 during exception handling.
    unsafe {
        core::arch::asm!("mrs {}, esr_el2", out(reg) esr);
        core::arch::asm!("mrs {}, far_el2", out(reg) far);
    }
    // ESR_EL2[31:26] = Exception Class; FAR_EL2 = faulting address for aborts (EC 0x20–0x25).
    panic!(
        "Synchronous exception at EL2 — ESR_EL2: {:#X}, FAR_EL2: {:#X}",
        esr, far
    );
}

#[unsafe(no_mangle)]
pub extern "C" fn EL2_IRQ_Handler() {
    // Spurious: no GIC initialized yet. Will become the IRQ dispatch point in Phase 3.
    crate::log_warn!("Spurious IRQ at EL2 — no GIC initialized");
}

#[unsafe(no_mangle)]
pub extern "C" fn EL2_FIQ_Handler() {
    // Spurious: FIQs at EL2 are unexpected before EL3 hands off GIC group 0.
    crate::log_warn!("Spurious FIQ at EL2 — unexpected before GIC group 0 handoff");
}

#[unsafe(no_mangle)]
pub extern "C" fn EL2_SError_Handler() {
    let esr: u64;
    // SAFETY: ESR_EL2 is always readable from EL2 during exception handling.
    unsafe {
        core::arch::asm!("mrs {}, esr_el2", out(reg) esr);
    }
    panic!("SError at EL2 — ESR_EL2: {:#X}", esr);
}
