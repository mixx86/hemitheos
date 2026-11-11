use crate::arch::arm::defs::mmu::{DIRECTMAP_PHYS_START, DIRECTMAP_VIRT_START, FIXMAP_ADDR};
use crate::arch::arm::defs::page::*;
use crate::common::page_alloc::alloc_page;
use crate::common::pmap::{pmap_map, pmap_unmap};
use crate::defs::errno::Errno;
use crate::defs::macros::*;
use crate::defs::mm::*;
use crate::defs::system::{SysState, system_state};
use crate::log_warn;
use crate::sync::SpinLock;
use core::arch::asm;
use core::cell::UnsafeCell;
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};

static PT_LOCK: SpinLock = SpinLock::new();

fn map_table(mfn: Mfn) -> Result<*mut Pte, Errno> {
    /* The directmap may not be set up yet during early boot */
    if system_state() == SysState::SYS_STATE_early_boot {
        return pmap_map(mfn);
    }

    Ok((DIRECTMAP_VIRT_START + ((mfn << PAGE_SHIFT) - DIRECTMAP_PHYS_START)) as *mut Pte)
}

fn unmap_table(table: *const Pte) {
    if system_state() == SysState::SYS_STATE_early_boot {
        pmap_unmap(table);
    }
}

struct TableGuard {
    ptr: *mut Pte,
}

impl TableGuard {
    pub fn new(root: Mfn) -> Result<Self, Errno> {
        Ok(Self {
            ptr: map_table(root)?,
        })
    }

    pub fn entry_ptr(&self, offset: usize) -> *mut Pte {
        debug_assert!(offset < PT_ENTRIES);
        unsafe { self.ptr.add(offset) }
    }

    pub fn entry(&self, offset: usize) -> Pte {
        unsafe { ptr::read_volatile(self.entry_ptr(offset)) }
    }

    pub fn remap(&mut self, mfn: Mfn) -> Result<(), Errno> {
        let new = map_table(mfn)?;
        unmap_table(self.ptr);
        self.ptr = new;
        Ok(())
    }
}

impl Drop for TableGuard {
    fn drop(&mut self) {
        unmap_table(self.ptr);
    }
}

fn set_fixmap(map: usize, mfn: Mfn, flags: Flags) {
    let res = map_pages(FIXMAP_ADDR(map), mfn, 1, flags);
    assert!(res.is_ok(), "set_fixmap({}) failed: {:?}", map, res);
}

fn clear_fixmap(map: usize) {
    let res = destroy_mappings(FIXMAP_ADDR(map), FIXMAP_ADDR(map) + PAGE_SIZE);
    assert!(res.is_ok(), "clear_fixmap({}) failed: {:?}", map, res);
}

pub(crate) fn write_pte(entry: *mut Pte, pte: Pte) {
    unsafe {
        asm!("dsb sy", options(nostack, preserves_flags));
        ptr::write_volatile(entry, pte);
        asm!("dsb sy", options(nostack, preserves_flags));
    }
}

pub(crate) fn isb() {
    unsafe { asm!("isb", options(nostack, preserves_flags)) };
}

fn flush_tlb_range_va(va: VAddr, size: u64) {
    let end = va + size;
    let mut va = va;

    unsafe {
        asm!("dsb sy", options(nostack, preserves_flags));
        while va < end {
            asm!("tlbi vae2is, {}", in(reg) va >> PAGE_SHIFT, options(nostack, preserves_flags));
            va += PAGE_SIZE;
        }
        asm!("dsb sy", "isb", options(nostack, preserves_flags));
    }
}

pub(crate) fn flush_tlb_range_va_local(va: VAddr, size: u64) {
    let end = va + size;
    let mut va = va;

    unsafe {
        asm!("dsb nshst", options(nostack, preserves_flags));
        while va < end {
            asm!("tlbi vae2, {}", in(reg) va >> PAGE_SHIFT, options(nostack, preserves_flags));
            va += PAGE_SIZE;
        }
        asm!("dsb nsh", "isb", options(nostack, preserves_flags));
    }
}

pub fn mfn_to_entry(mfn: Mfn, attr: u32) -> Pte {
    /* AP[1] (PTE_UP) is RES1 at EL2 */
    let mut e = Pte {
        raw: PTE_VALID
            | ((attr as u64) << PTE_AI_SHIFT) & PTE_AI_MASK
            | PTE_NS
            | PTE_UP
            | PTE_AF
            | PTE_NG
            | PTE_XN,
    };

    let sh = match attr {
        /*
         * Normal Inner/Outer Non-cacheable and Device memory are treated as
         * Outer Shareable regardless of the SH field.
         * ARM DDI 0487 M.b, D8.6.2 "Stage 1 Shareability attributes", RPYFVQ.
         * With TCR_EL2.DS == 1 (FEAT_LPA2) bits[9:8] are OA[51:50], not SH
         * (same section, IHRNGF).
         */
        MT_NORMAL_NC | MT_DEVICE_nGnRnE | MT_DEVICE_nGnRE => LPAE_SH_OUTER,
        _ => LPAE_SH_INNER,
    };
    e.set_sh(sh);

    debug_assert!((mfn << PAGE_SHIFT) & !PADDR_MASK == 0);
    e.set_mfn(mfn);

    e
}

const EARLY_POOL_PAGES: usize = 8;

#[repr(C, align(4096))]
struct EarlyPool([[u8; PAGE_SIZE as usize]; EARLY_POOL_PAGES]);

static mut EARLY_POOL: EarlyPool = EarlyPool([[0; PAGE_SIZE as usize]; EARLY_POOL_PAGES]);
static EARLY_POOL_NEXT: AtomicUsize = AtomicUsize::new(0);

fn alloc_table() -> Result<Mfn, Errno> {
    if system_state() != SysState::SYS_STATE_early_boot {
        return alloc_page();
    }

    let idx = EARLY_POOL_NEXT.fetch_add(1, Ordering::Relaxed);
    if idx >= EARLY_POOL_PAGES {
        return Err(Errno::HEMITHEOS_ENOMEM);
    }

    let va = unsafe { &raw mut EARLY_POOL.0[idx] } as VAddr;
    Ok(get_afn(virt_to_maddr(va)?))
}

pub(crate) fn virt_to_maddr(va: VAddr) -> Result<MAddr, Errno> {
    let par: u64;
    unsafe {
        asm!(
            "at s1e2r, {va}",
            "isb",
            "mrs {par}, PAR_EL1",
            va = in(reg) va,
            par = out(reg) par,
            options(nostack, preserves_flags),
        );
    }

    if par & 1 != 0 {
        return Err(Errno::HEMITHEOS_EINVAL);
    }

    Ok((par & PTE_ADDR_MASK) | (va & (PAGE_SIZE - 1)))
}

fn create_table(entry: *mut Pte) -> Result<(), Errno> {
    let mfn = alloc_table()?;

    let p = map_table(mfn)?;
    unsafe { ptr::write_bytes(p as *mut u8, 0, PAGE_SIZE as usize) };
    unmap_table(p);

    let mut pte = mfn_to_entry(mfn, MT_NORMAL);
    pte.set_bit(PTE_TABLE, true);
    write_pte(entry, pte);
    /* No ISB here, deferred to pt_update() */

    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NextLevel {
    Normal,
    SuperPage,
}

fn pt_next_level(
    im_not_mapping_that: bool,
    level: usize,
    table: &mut TableGuard,
    offset: usize,
) -> Result<NextLevel, Errno> {
    debug_assert!(level < 3);

    let entry_ptr = table.entry_ptr(offset);

    if !table.entry(offset).is_valid() {
        if im_not_mapping_that {
            return Err(Errno::HEMITHEOS_TABLE_MAP_FAILED);
        }

        if create_table(entry_ptr).is_err() {
            return Err(Errno::HEMITHEOS_TABLE_MAP_FAILED);
        }
    }

    let entry = table.entry(offset);

    if entry.is_mapping(level as u8) {
        return Ok(NextLevel::SuperPage);
    }

    table.remap(entry.mfn())?;

    Ok(NextLevel::Normal)
}

fn pt_check_entry(entry: Pte, mfn: Mfn, level: usize, flags: Flags) -> bool {
    let level = level as u8;

    if flags.contains(_PAGE_PRESENT) && mfn_eq(mfn, INVALID_MFN) {
        if !entry.is_valid() {
            log_warn!("Modifying invalid entry is not allowed.\n");
            return false;
        }

        if !entry.is_mapping(level) {
            log_warn!("Modifying a table entry is not allowed.\n");
            return false;
        }

        if entry.ai() != PAGE_AI_MASK(flags.bits()) {
            log_warn!(
                "Modifying memory attributes is not allowed ({:#x} -> {:#x}).\n",
                entry.ai(),
                PAGE_AI_MASK(flags.bits())
            );
            return false;
        }

        if entry.contig() {
            log_warn!("Modifying entry with contiguous bit set is not allowed.\n");
            return false;
        }
    } else if flags.contains(_PAGE_PRESENT) {
        debug_assert!(!mfn_eq(mfn, INVALID_MFN));

        /* pt_update() relies on this to skip the TLB flush */
        if entry.is_valid() {
            if entry.is_mapping(level) {
                log_warn!(
                    "Changing MFN for a valid entry is not allowed ({:#x} -> {:#x}).\n",
                    entry.mfn(),
                    mfn
                );
            } else {
                log_warn!("Trying to replace a table with a mapping.\n");
            }
            return false;
        }
    } else if !flags.intersects(_PAGE_PRESENT | _PAGE_POPULATE) {
        debug_assert!(mfn_eq(mfn, INVALID_MFN));

        if entry.is_table(level) {
            log_warn!("Removing a table is not allowed.\n");
            return false;
        }

        if entry.contig() {
            log_warn!("Removing entry with contiguous bit set is not allowed.\n");
            return false;
        }
    } else {
        debug_assert!(flags.contains(_PAGE_POPULATE));
        debug_assert!(mfn_eq(mfn, INVALID_MFN));
    }

    true
}

fn pt_update_entry(
    root: Mfn,
    virt: VAddr,
    mfn: Mfn,
    target: usize,
    flags: Flags,
) -> Result<(), Errno> {
    /*
     * For missing tables below lvl.3: Aborts if both the MFN and populate bit
     * are invalid. However, if the MFN is valid, it forces auto-population
     * regardless of the populate bit, freeing the caller from intermediate table management.
     */
    let im_not_mapping_that = mfn_eq(mfn, INVALID_MFN) && !flags.contains(_PAGE_POPULATE);
    let offsets = EXTRACT_OFFSETS_FROM_ADDR(virt);

    debug_assert!(!flags.contains(_PAGE_POPULATE | _PAGE_PRESENT));

    let mut table = TableGuard::new(root)?;
    let mut level = HYP_PT_ROOT_LEVEL as usize;

    while level < target {
        match pt_next_level(im_not_mapping_that, level, &mut table, offsets[level]) {
            Ok(NextLevel::Normal) => level += 1,
            Ok(NextLevel::SuperPage) => break,
            Err(Errno::HEMITHEOS_TABLE_MAP_FAILED) => {
                /* Fine when removing, since the mapping may not exist at all */
                if flags.intersects(_PAGE_PRESENT | _PAGE_POPULATE) {
                    log_warn!("pt_update_entry: Unable to map level {}\n", level);
                    return Err(Errno::HEMITHEOS_ENOENT);
                }
                return Ok(());
            }
            Err(err) => return Err(err),
        }
    }

    if level != target {
        log_warn!("pt_update_entry: Shattering superpage is not supported\n");
        return Err(Errno::HEMITHEOS_EOPNOTSUPP);
    }

    let entry_ptr = table.entry_ptr(offsets[level]);
    let entry = table.entry(offsets[level]);

    if !pt_check_entry(entry, mfn, level, flags) {
        return Err(Errno::HEMITHEOS_EINVAL);
    }

    if flags.contains(_PAGE_POPULATE) {
        return Ok(());
    }

    let pte = if !flags.contains(_PAGE_PRESENT) {
        Pte::EMPTY
    } else {
        let mut pte = if !mfn_eq(mfn, INVALID_MFN) {
            let mut pte = mfn_to_entry(mfn, PAGE_AI_MASK(flags.bits()));
            /* Blocks (lvl 1/2) have table = 0, pages (lvl 3) table = 1 */
            pte.set_bit(PTE_TABLE, level == 3);
            pte
        } else {
            entry
        };

        pte.set_bit(PTE_RO, PAGE_RO_MASK(flags.bits()) != 0);
        pte.set_bit(PTE_XN, PAGE_XN_MASK(flags.bits()) != 0);
        pte.set_bit(PTE_CONTIG, flags.contains(_PAGE_CONTIG));
        pte
    };

    write_pte(entry_ptr, pte);
    /* No ISB or TLB flush here, deferred to pt_update() */

    Ok(())
}

fn pt_mapping_level(vfn: Vfn, mfn: Mfn, nr: u64, flags: Flags) -> usize {
    let mut mask = if !mfn_eq(mfn, INVALID_MFN) { mfn } else { 0 };
    mask |= vfn;

    if !flags.contains(_PAGE_BLOCK) {
        3
    } else if is_lvl_one(mask, nr) {
        1
    } else if is_lvl_two(mask, nr) {
        2
    } else {
        3
    }
}

const PT_4K_NR_CONTIG: u64 = 16;

fn pt_check_contig(vfn: Vfn, mfn: Mfn, level: usize, left: u64, flags: Flags) -> u64 {
    if !flags.contains(_PAGE_BLOCK) {
        return 1;
    }

    if mfn_eq(mfn, INVALID_MFN) {
        return 1;
    }

    const _: () = assert!(PAGE_SIZE == KB(4));

    let nr_contig = BIT(PT_LEVEL_ORDER(level as u64)) * PT_4K_NR_CONTIG;

    if left < nr_contig || ((mfn | vfn) & (nr_contig - 1)) != 0 {
        return 1;
    }

    PT_4K_NR_CONTIG
}

fn pt_update_locked(
    root: Mfn,
    vfn: Vfn,
    mfn: Mfn,
    nr_mfns: u64,
    flags: Flags,
) -> Result<(), Errno> {
    let mut vfn = vfn;
    let mut mfn = mfn;
    let mut left = nr_mfns;

    while left > 0 {
        let level = pt_mapping_level(vfn, mfn, left, flags);
        let order = PT_LEVEL_ORDER(level as u64);

        debug_assert!(left >= BIT(order));

        let nr_contig = pt_check_contig(vfn, mfn, level, left, flags);
        let new_flags = if nr_contig > 1 {
            flags | _PAGE_CONTIG
        } else {
            flags
        };

        for _ in 0..nr_contig {
            pt_update_entry(root, vfn << PAGE_SHIFT, mfn, level, new_flags)?;

            vfn += BIT(order);
            if !mfn_eq(mfn, INVALID_MFN) {
                mfn += BIT(order);
            }

            left -= BIT(order);
        }
    }

    Ok(())
}

pub fn pt_update(virt: VAddr, mfn: Mfn, nr_mfns: u64, flags: Flags) -> Result<(), Errno> {
    let mut root: Mfn;
    unsafe {
        asm!("mrs {}, TTBR0_EL2", out(reg) root, options(nomem, nostack, preserves_flags));
    }
    root = get_afn(root & PTE_ADDR_MASK);

    if flags.has_rwx() {
        log_warn!("Mappings should not be both Writeable and Executable.\n");
        return Err(Errno::HEMITHEOS_EINVAL);
    }

    if flags.contains(_PAGE_CONTIG) {
        log_warn!("_PAGE_CONTIG is an internal only flag.\n");
        return Err(Errno::HEMITHEOS_EINVAL);
    }

    if !is_aligned(virt, PAGE_SIZE) {
        log_warn!("The virtual address is not aligned to the page-size.\n");
        return Err(Errno::HEMITHEOS_EINVAL);
    }

    PT_LOCK.lock();

    let rc = pt_update_locked(root, get_afn(virt), mfn, nr_mfns, flags);

    /*
     * Inserting never replaces a valid entry (see pt_check_entry()), so no
     * flush is needed, only an ISB for write_pte()'s DSB. Otherwise flush
     * even on failure: the table may have been partially modified.
     */
    if !flags.contains(_PAGE_PRESENT) || mfn_eq(mfn, INVALID_MFN) {
        flush_tlb_range_va(virt, PAGE_SIZE * nr_mfns);
    } else {
        isb();
    }

    PT_LOCK.unlock();

    rc
}

pub fn map_pages(virt: VAddr, mfn: Mfn, nr_mfns: u64, flags: Flags) -> Result<(), Errno> {
    pt_update(virt, mfn, nr_mfns, flags)
}

pub fn populate_pt_range(virt: VAddr, nr_mfns: u64) -> Result<(), Errno> {
    pt_update(virt, INVALID_MFN, nr_mfns, Flags(_PAGE_POPULATE))
}

pub fn destroy_mappings(s: VAddr, e: VAddr) -> Result<(), Errno> {
    debug_assert!(is_aligned(s, PAGE_SIZE));
    debug_assert!(is_aligned(e, PAGE_SIZE));
    debug_assert!(s <= e);
    pt_update(s, INVALID_MFN, (e - s) >> PAGE_SHIFT, Flags(0))
}

pub fn modify_mappings(s: VAddr, e: VAddr, nf: Flags) -> Result<(), Errno> {
    debug_assert!(is_aligned(s, PAGE_SIZE));
    debug_assert!(is_aligned(e, PAGE_SIZE));
    debug_assert!(s <= e);
    pt_update(s, INVALID_MFN, (e - s) >> PAGE_SHIFT, nf)
}
