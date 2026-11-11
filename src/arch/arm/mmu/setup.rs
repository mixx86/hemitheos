use crate::arch::arm::defs::config::{BOOT_INFO_VIRT_START};
use crate::defs::mm::{MAddr, get_afn};
use crate::defs::macros::MB;
use crate::arch::arm::pt::map_pages;
use crate::arch::defs::*;


#[unsafe(no_mangle)]
pub extern "C" fn early_boot_info_map(paddr: MAddr)
{
    debug_assert!(paddr & (MB(2) - 1) == 0);
    let flags = Flags(_PAGE_PRESENT | MT_NORMAL | _PAGE_RO | _PAGE_XN | _PAGE_BLOCK);
    match map_pages(BOOT_INFO_VIRT_START, get_afn(paddr), MB(2) >> PAGE_SHIFT, flags) {
        Ok(()) => {},
        Err(errno) => panic!("early_boot_info_map failed: {:?}", errno),
    }
}

pub fn setup_directmap_mappings(base: MAddr, size: u64) {
    let virt = DIRECTMAP_VIRT_START + (base - DIRECTMAP_PHYS_START);
    let flags = Flags(_PAGE_PRESENT | MT_NORMAL | _PAGE_XN | _PAGE_BLOCK);

    if let Err(errno) = map_pages(virt, get_afn(base), size >> PAGE_SHIFT, flags) {
        panic!("setup_directmap_mappings failed: {:?}", errno);
    }
}
