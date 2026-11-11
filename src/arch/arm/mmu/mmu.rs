use crate::arch::defs::*;
use crate::common::vmap::vmap;
use crate::defs::errno::Errno;
use crate::defs::mm::{MAddr, VAddr, get_afn};
use core::arch::asm;

pub fn map_device(_addr: u64) {}

pub fn ioremap(paddr: MAddr, size: u64) -> Result<VAddr, Errno> {
    let offset = paddr & (PAGE_SIZE - 1);
    let nr = (offset + size + PAGE_SIZE - 1) >> PAGE_SHIFT;
    let flags = Flags(_PAGE_PRESENT | MT_DEVICE_nGnRE | _PAGE_XN);

    let va = vmap(get_afn(paddr), nr, flags)?;

    Ok(va + offset)
}
