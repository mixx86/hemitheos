/*
 * PMAP: temporary mappings through dedicated fixmap slots.
 *
 * During early boot the directmap may not be set up yet, so page tables are
 * temporarily mapped through the FIX_PMAP_* fixmap slots.
 */
use crate::arch::arm::defs::fixmap::{FIX_PMAP_BEGIN, FIX_PMAP_END, NUM_FIX_PMAP};
use crate::arch::arm::defs::mmu::FIXMAP_ADDR;
use crate::arch::arm::defs::page::{PAGE_SHIFT, PAGE_SIZE};
use crate::arch::arm::mmu::pmap::{arch_pmap_map, arch_pmap_unmap};
use crate::defs::errno::Errno;
use crate::defs::mm::*;
use crate::defs::system::{SysState, system_state};
use crate::log_warn;
use core::sync::atomic::{AtomicU32, Ordering};

const _: () = assert!(NUM_FIX_PMAP <= 32);

/* Bit n set = slot FIX_PMAP_BEGIN + n in use */
static PMAP_INUSE: AtomicU32 = AtomicU32::new(0);

pub fn pmap_map(mfn: Mfn) -> Result<*mut Pte, Errno> {
    debug_assert!(system_state() == SysState::SYS_STATE_early_boot);

    let mut inuse = PMAP_INUSE.load(Ordering::Relaxed);
    let idx = loop {
        let idx = (!inuse).trailing_zeros() as usize;
        if idx >= NUM_FIX_PMAP {
            log_warn!("pmap_map: no free PMAP slot.\n");
            return Err(Errno::HEMITHEOS_ENOMEM);
        }

        match PMAP_INUSE.compare_exchange(
            inuse,
            inuse | (1 << idx),
            Ordering::Acquire,
            Ordering::Relaxed,
        ) {
            Ok(_) => break idx,
            Err(cur) => inuse = cur,
        }
    };

    let slot = FIX_PMAP_BEGIN + idx;
    arch_pmap_map(slot, mfn);

    Ok(FIXMAP_ADDR(slot) as *mut Pte)
}

pub fn pmap_unmap(p: *const Pte) {
    let va = (p as VAddr) & !(PAGE_SIZE - 1);
    let slot = ((va - FIXMAP_ADDR(0)) >> PAGE_SHIFT) as usize;

    debug_assert!((FIX_PMAP_BEGIN..=FIX_PMAP_END).contains(&slot));

    arch_pmap_unmap(slot);
    PMAP_INUSE.fetch_and(!(1 << (slot - FIX_PMAP_BEGIN)), Ordering::Release);
}
