use crate::arch::defs::PAGE_SIZE;
use crate::defs::errno::Errno;
use crate::defs::mm::{MAddr, Mfn, get_afn};
use core::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);
static END: AtomicU64 = AtomicU64::new(0);
static RESERVED_START: AtomicU64 = AtomicU64::new(0);
static RESERVED_END: AtomicU64 = AtomicU64::new(0);

fn skip_reserved(addr: MAddr) -> MAddr {
    let start = RESERVED_START.load(Ordering::Relaxed);
    let end = RESERVED_END.load(Ordering::Relaxed);

    if addr >= start && addr < end { end } else { addr }
}

pub fn init(start: MAddr, end: MAddr, reserved_start: MAddr, reserved_end: MAddr) {
    RESERVED_START.store(reserved_start & !(PAGE_SIZE - 1), Ordering::Relaxed);
    RESERVED_END.store((reserved_end + PAGE_SIZE - 1) & !(PAGE_SIZE - 1), Ordering::Relaxed);
    END.store(end & !(PAGE_SIZE - 1), Ordering::Relaxed);
    NEXT.store((start + PAGE_SIZE - 1) & !(PAGE_SIZE - 1), Ordering::Release);
}

pub fn alloc_page() -> Result<Mfn, Errno> {
    let end = END.load(Ordering::Relaxed);

    let prev = NEXT
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |next| {
            let page = skip_reserved(next);
            if page + PAGE_SIZE > end { None } else { Some(page + PAGE_SIZE) }
        })
        .map_err(|_| Errno::HEMITHEOS_ENOMEM)?;

    Ok(get_afn(skip_reserved(prev)))
}
