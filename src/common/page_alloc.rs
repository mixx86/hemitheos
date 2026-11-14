use crate::arch::defs::PAGE_SIZE;
use crate::defs::errno::Errno;
use crate::defs::mm::{MAX_MEM_REGIONS, MemRegion, MAddr, Mfn, get_afn};
use core::sync::atomic::{AtomicUsize, AtomicU64, Ordering};

struct Region {
  next: AtomicU64,
  end: AtomicU64,
}
static REGIONS: [Region; MAX_MEM_REGIONS] = [const {
    Region {
        next: AtomicU64::new(0),
        end:   AtomicU64::new(0),
    }
}; MAX_MEM_REGIONS];
static NR_REGIONS: AtomicUsize = AtomicUsize::new(0);

pub fn init<'a>(regions: impl Iterator<Item = &'a MemRegion>)
{
    let mut nr = 0;
    for r in regions {
        let start = (r.start + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        let end = r.end & !(PAGE_SIZE - 1);

        if start >= end {
            continue;
        }

        assert!(nr < MAX_MEM_REGIONS, "page_alloc: too many regions");
        REGIONS[nr].end.store(end, Ordering::Relaxed);
        REGIONS[nr].next.store(start, Ordering::Relaxed);
        nr += 1;
    }

    NR_REGIONS.store(nr, Ordering::Release);
}

pub fn alloc_page() -> Result<Mfn, Errno> {
    let nr = NR_REGIONS.load(Ordering::Acquire);

    for region in &REGIONS[..nr] {
        let end = region.end.load(Ordering::Relaxed);
        let page = region.next.fetch_update(
            Ordering::AcqRel, Ordering::Acquire, |next|
            {
                if next + PAGE_SIZE > end { None } else { Some(next + PAGE_SIZE) }
            }
        );
        if let Ok(page) = page {
            return Ok(get_afn(page));
        }
    }

    Err(Errno::HEMITHEOS_ENOMEM)
}
