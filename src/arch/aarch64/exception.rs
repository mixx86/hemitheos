#[unsafe(no_mangle)]
pub extern "C" fn el2_sync_handler() {
    let esr: u64;
    unsafe {
        core::arch::asm!("mrs {}, esr_el2", out(reg) esr);
    }
    panic!("Synchronous Exception at EL2! ESR_EL2: {:#X}", esr);
}
