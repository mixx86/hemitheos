use crate::arch::arm::defs::config::*;
use crate::defs::macros::KB;

pub const FIXMAP_VIRT_START: u64 = HEMITHEOS_VIRT_START + HEMITHEOS_VIRT_SIZE;
pub const fn FIXMAP_ADDR(n: usize) -> u64 {
    FIXMAP_VIRT_START + n as u64 + KB(4)
}
