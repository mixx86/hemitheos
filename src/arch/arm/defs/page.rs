use crate::defs::macros::BIT;
use core::ops::{BitOr, BitOrAssign};

/* Page table geometry */
pub const PAGE_SHIFT: u64 = 12;
pub const PAGE_SIZE: u64 = 1 << PAGE_SHIFT;
pub const PT_INDEX_BITS: u64 = 9;

pub const L0_BLOCK_ADDR_MASK: u64 = 0xFFFF_FF80_0000_0000;
pub const L1_BLOCK_ADDR_MASK: u64 = 0xFFFF_FFFF_C000_0000;
pub const L2_BLOCK_ADDR_MASK: u64 = 0xFFFF_FFFF_FFE0_0000;
pub const L3_BLOCK_ADDR_MASK: u64 = 0xFFFF_FFFF_FFFF_F000;

pub const HYP_PT_ROOT_LEVEL: u8 = 0;

pub const fn EXTRACT_OFFSETS_FROM_ADDR(addr: u64) -> [usize; 4] {
    [
        ((addr >> PT_LEVEL_SHIFT(0)) & 0x1FF) as usize,
        ((addr >> PT_LEVEL_SHIFT(1)) & 0x1FF) as usize,
        ((addr >> PT_LEVEL_SHIFT(2)) & 0x1FF) as usize,
        ((addr >> PT_LEVEL_SHIFT(3)) & 0x1FF) as usize,
    ]
}

const fn PT_LEVEL_SHIFT(lvl: u64) -> u64 {
    PAGE_SHIFT + PT_INDEX_BITS * (3 - lvl)
}

pub const fn PT_LEVEL_ORDER(lvl: u64) -> u64 {
    PT_INDEX_BITS * (3 - lvl)
}

pub const SECOND_ORDER: u64 = PT_LEVEL_ORDER(2);

pub const FIRST_ORDER: u64 = PT_LEVEL_ORDER(1);

/* Page table flags */
/*
 * Layout of the flags used for updating the hypervisor page tables
 *
 * [0:2] Memory Attribute Index
 * [3:4] Permission flags
 * [5]   Page present
 * [6]   Only populate page tables
 * [7]   Superpage mappings is allowed
 * [8]   Set contiguous bit (internal flag)
*/
pub const _PAGE_XN_BIT: u8 = 3;
pub const _PAGE_RO_BIT: u8 = 4;

pub const _PAGE_XN: u32 = 1 << _PAGE_XN_BIT;
pub const _PAGE_RO: u32 = 1 << _PAGE_RO_BIT;

pub const _PAGE_PRESENT_BIT: u8 = 5;
pub const _PAGE_PRESENT: u32 = 1 << _PAGE_PRESENT_BIT;

pub const _PAGE_POPULATE_BIT: u8 = 6;
pub const _PAGE_POPULATE: u32 = 1 << _PAGE_POPULATE_BIT;

pub const _PAGE_BLOCK_BIT: u8 = 7;
pub const _PAGE_BLOCK: u32 = 1 << _PAGE_BLOCK_BIT;

pub const _PAGE_CONTIG_BIT: u8 = 8;
pub const _PAGE_CONTIG: u32 = 1 << _PAGE_CONTIG_BIT;

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Flags(pub u32);

impl Flags {
    pub fn contains(&self, mask: u32) -> bool {
        (self.0 & mask) == mask
    }

    pub fn intersects(&self, mask: u32) -> bool {
        (self.0 & mask) != 0
    }

    pub fn has_rwx(&self) -> bool {
        if self.contains(_PAGE_PRESENT) && !self.contains(_PAGE_XN) && !self.contains(_PAGE_RO) {
            return true;
        }
        false
    }
}

impl BitOr<u32> for Flags {
    type Output = Flags;

    fn bitor(self, rhs: u32) -> Self::Output {
        Flags(self.0 | rhs)
    }
}

impl BitOr for Flags {
    type Output = Flags;

    fn bitor(self, rhs: Flags) -> Self::Output {
        Flags(self.0 | rhs.0)
    }
}

impl BitOrAssign<u32> for Flags {
    fn bitor_assign(&mut self, rhs: u32) {
        self.0 |= rhs;
    }
}

#[cold]
#[inline(never)]
pub fn is_lvl_one(mask: u64, nr: u64) -> bool {
    return mask & (BIT(FIRST_ORDER) - 1) == 0 && nr >= BIT(FIRST_ORDER);
}

#[cold]
#[inline(never)]
pub fn is_lvl_two(mask: u64, nr: u64) -> bool {
    return mask & (BIT(SECOND_ORDER) - 1) == 0 && nr >= BIT(SECOND_ORDER);
}
