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

pub const fn get_afn(addr: u64) -> u64 {
    return addr >> PAGE_SHIFT;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(transparent)]
pub struct Pte {
    pub raw: u64,
}

impl Pte {
    pub const EMPTY: Pte = Pte { raw: 0 };

    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        self.raw & PTE_VALID != 0
    }

    #[inline(always)]
    pub fn kind(&self, level: u8) -> PteKind {
        match (self.raw & (PTE_VALID | PTE_TABLE), level) {
            (0b01, 0..=2) => PteKind::SuperBlock,
            (0b11, 0..=2) => PteKind::Table,
            (0b11, 3) => PteKind::Block,
            _ => PteKind::Invalid,
        }
    }

    /* Valid entry that maps memory (block or page) */
    #[inline(always)]
    pub fn is_mapping(&self, level: u8) -> bool {
        matches!(self.kind(level), PteKind::SuperBlock | PteKind::Block)
    }

    /* Valid entry that points to a next-level table */
    #[inline(always)]
    pub fn is_table(&self, level: u8) -> bool {
        self.kind(level) == PteKind::Table
    }

    #[inline(always)]
    pub fn mfn(&self) -> Mfn {
        get_afn(self.raw & PTE_ADDR_MASK)
    }

    #[inline(always)]
    pub fn set_mfn(&mut self, mfn: Mfn) {
        self.raw = (self.raw & !PTE_ADDR_MASK) | ((mfn << PAGE_SHIFT) & PTE_ADDR_MASK);
    }

    #[inline(always)]
    pub fn ai(&self) -> u32 {
        ((self.raw & PTE_AI_MASK) >> PTE_AI_SHIFT) as u32
    }

    #[inline(always)]
    pub fn contig(&self) -> bool {
        self.raw & PTE_CONTIG != 0
    }

    #[inline(always)]
    pub fn set_sh(&mut self, sh: u64) {
        self.raw = (self.raw & !PTE_SH_MASK) | ((sh << PTE_SH_SHIFT) & PTE_SH_MASK);
    }

    #[inline(always)]
    pub fn set_bit(&mut self, bit: u64, on: bool) {
        if on {
            self.raw |= bit;
        } else {
            self.raw &= !bit;
        }
    }
}
