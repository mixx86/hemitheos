// EL2 exception handlers — called from the entry stubs in exception_entry.s.
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
    unsafe {
        core::arch::asm!("mrs {}, esr_el2", out(reg) esr);
    }
    panic!("Synchronous exception at EL2 — ESR_EL2: {:#X}", esr);
}

#[unsafe(no_mangle)]
pub extern "C" fn EL2_IRQ_Handler() {
    // Spurious: no GIC initialized yet. Will become the IRQ dispatch point in Phase 2.
}

#[unsafe(no_mangle)]
pub extern "C" fn EL2_FIQ_Handler() {
    // Spurious: FIQs at EL2 are unexpected before EL3 hands off GIC group 0.
}

#[unsafe(no_mangle)]
pub extern "C" fn EL2_SError_Handler() {
    let esr: u64;
    unsafe {
        core::arch::asm!("mrs {}, esr_el2", out(reg) esr);
    }
    panic!("SError at EL2 — ESR_EL2: {:#X}", esr);
}
