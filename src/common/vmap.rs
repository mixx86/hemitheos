use crate::arch::defs::{Flags, PAGE_SIZE, VMAP_VIRT_SIZE, VMAP_VIRT_START};
use crate::arch::mmu::pt::map_pages;
use crate::defs::errno::Errno;
use crate::defs::mm::{Mfn, VAddr};
use core::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(VMAP_VIRT_START);

pub fn vmap(mfn: Mfn, nr: u64, flags: Flags) -> Result<VAddr, Errno> {
    if nr == 0 {
        return Err(Errno::HEMITHEOS_EINVAL);
    }

    let size = nr * PAGE_SIZE;

    let va = NEXT
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |next| {
            if next + size > VMAP_VIRT_START + VMAP_VIRT_SIZE {
                None
            } else {
                Some(next + size)
            }
        })
        .map_err(|_| Errno::HEMITHEOS_ENOMEM)?;

    map_pages(va, mfn, nr, flags)?;

    Ok(va)
}
