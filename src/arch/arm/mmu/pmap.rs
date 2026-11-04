use super::pt::{flush_tlb_range_va_local, isb, mfn_to_entry, write_pte};
use crate::arch::arm::defs::fixmap::FIX_PMAP_END;
use crate::arch::arm::defs::mmu::{FIXMAP_ADDR, FIXMAP_VIRT_START};
use crate::arch::arm::defs::page::*;
use crate::defs::macros::MB;
use crate::defs::mm::*;
use core::ptr;

unsafe extern "C" {
    static mut boot_fixmap: [Pte; PT_ENTRIES];
}

/* boot_fixmap[slot] must map FIXMAP_ADDR(slot) */
const _: () = assert!(FIXMAP_VIRT_START & (MB(2) - 1) == 0);
const _: () = assert!(FIX_PMAP_END < PT_ENTRIES);

fn fixmap_entry(slot: usize) -> *mut Pte {
    unsafe { (&raw mut boot_fixmap as *mut Pte).add(slot) }
}

pub fn arch_pmap_map(slot: usize, mfn: Mfn) {
    let entry = fixmap_entry(slot);

    debug_assert!(!unsafe { ptr::read_volatile(entry) }.is_valid());

    let mut pte = mfn_to_entry(mfn, MT_NORMAL);
    pte.set_bit(PTE_TABLE, true);
    write_pte(entry, pte);
    isb();
}

pub fn arch_pmap_unmap(slot: usize) {
    write_pte(fixmap_entry(slot), Pte::EMPTY);
    flush_tlb_range_va_local(FIXMAP_ADDR(slot), PAGE_SIZE);
}
