use core::sync::atomic::{AtomicU8, Ordering};

#[allow(non_camel_case_types)]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SysState {
    SYS_STATE_early_boot = 0,
    SYS_STATE_boot = 1,
    SYS_STATE_smp_boot = 2,
    SYS_STATE_active = 3,
}

static SYSTEM_STATE: AtomicU8 = AtomicU8::new(SysState::SYS_STATE_early_boot as u8);

pub fn system_state() -> SysState {
    match SYSTEM_STATE.load(Ordering::Acquire) {
        0 => SysState::SYS_STATE_early_boot,
        1 => SysState::SYS_STATE_boot,
        2 => SysState::SYS_STATE_smp_boot,
        _ => SysState::SYS_STATE_active,
    }
}

pub fn set_system_state(state: SysState) {
    SYSTEM_STATE.store(state as u8, Ordering::Release);
}
