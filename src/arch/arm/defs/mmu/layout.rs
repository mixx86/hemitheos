use crate::arch::arm::defs::config::*;
use crate::defs::macros::KB;

pub const FIXMAP_VIRT_START: u64 = HEMITHEOS_VIRT_START + HEMITHEOS_VIRT_SIZE;
pub const fn FIXMAP_ADDR(n: usize) -> u64 {
    FIXMAP_VIRT_START + n as u64 * KB(4)
}

pub const DIRECTMAP_VIRT_START: u64 = 0x0000_8000_0000_0000;
pub const DIRECTMAP_PHYS_START: u64 = 0x4000_0000;
