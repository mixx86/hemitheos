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

pub const PT_ENTRIES: usize = 1 << PT_INDEX_BITS;

pub const MT_DEVICE_nGnRnE: u32 = 0x0;
pub const MT_NORMAL_NC: u32 = 0x1;
pub const MT_NORMAL_WT: u32 = 0x2;
pub const MT_NORMAL_WB: u32 = 0x3;
pub const MT_DEVICE_nGnRE: u32 = 0x4;
pub const MT_NORMAL: u32 = 0x7;

/* Physical address width supported by the page-table code */
pub const PADDR_BITS: u64 = 48;
pub const PADDR_MASK: u64 = (1 << PADDR_BITS) - 1;

/*
 * VMSAv8-64 stage-1 descriptor bits (4KB granule).
 *
 * [0]     Valid
 * [1]     Table (lvl 0-2) / Page (lvl 3)
 * [4:2]   AttrIndx
 * [5]     NS
 * [7:6]   AP[2:1]
 * [9:8]   SH
 * [10]    AF
 * [11]    nG
 * [47:12] Output address
 * [52]    Contiguous
 * [54]    XN
 * [58:55] Software use
 */
pub const PTE_VALID: u64 = 1 << 0;
pub const PTE_TABLE: u64 = 1 << 1;
pub const PTE_AI_SHIFT: u64 = 2;
pub const PTE_AI_MASK: u64 = 0x7 << PTE_AI_SHIFT;
pub const PTE_NS: u64 = 1 << 5;
pub const PTE_UP: u64 = 1 << 6; /* AP[1], RES1 at EL2 */
pub const PTE_RO: u64 = 1 << 7; /* AP[2] */
pub const PTE_SH_SHIFT: u64 = 8;
pub const PTE_SH_MASK: u64 = 0x3 << PTE_SH_SHIFT;
pub const PTE_AF: u64 = 1 << 10;
pub const PTE_NG: u64 = 1 << 11;
pub const PTE_ADDR_MASK: u64 = PADDR_MASK & !(PAGE_SIZE - 1);
pub const PTE_CONTIG: u64 = 1 << 52;
pub const PTE_XN: u64 = 1 << 54;

/* Shareability */
pub const LPAE_SH_NON: u64 = 0x0;
pub const LPAE_SH_OUTER: u64 = 0x2;
pub const LPAE_SH_INNER: u64 = 0x3;

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

pub const fn PAGE_AI_MASK(x: u32) -> u32 {
    x & 0x7
}

pub const fn PAGE_XN_MASK(x: u32) -> u32 {
    (x >> _PAGE_XN_BIT) & 0x1
}

pub const fn PAGE_RO_MASK(x: u32) -> u32 {
    (x >> _PAGE_RO_BIT) & 0x1
}

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
pub struct Flags(pub u32);

impl Flags {
    pub const fn bits(&self) -> u32 {
        self.0
    }

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
