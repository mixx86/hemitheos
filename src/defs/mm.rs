use crate::arch::arm::defs::*;

pub type MAddr = u64;
pub type VAddr = u64;
pub type Mfn = u64;
pub type Vfn = u64;

pub const INVALID_MFN: Mfn = !0;
#[inline(always)]
pub fn mfn_eq(x: Mfn, y: Mfn) -> bool {
    return x == y;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PteKind {
    SuperBlock,
    Table,
    Block,
    Invalid,
}

/*const fn frame_base(raw: u64, level: u8) -> u64 {
    match level {
        0 => raw & L0_BLOCK_ADDR_MASK,
        1 => raw & L1_BLOCK_ADDR_MASK,
        2 => raw & L2_BLOCK_ADDR_MASK,
        3 => raw & L3_BLOCK_ADDR_MASK,
        _ => panic!("invalid page table level"),
    }
}*/

pub const fn get_afn(addr: u64) -> u64 {
    return addr >> PAGE_SHIFT;
}

pub struct Pte {
    pub raw: u64,
}

impl Pte {
    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        return self.raw & 0b1 != 0;
    }

    #[inline(always)]
    pub fn kind(&self, level: u8) -> PteKind {
        match self.raw & 0b11 {
            0b01 if level < 3 => PteKind::SuperBlock,
            0b11 => PteKind::Table,
            0b01 => PteKind::Block,
            _ => PteKind::Invalid,
        }
    }

    #[inline(always)]
    pub fn mfn(&self) -> Mfn {
        get_afn(self.raw)
    }
}
