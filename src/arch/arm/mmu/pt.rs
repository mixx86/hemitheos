use crate::arch::arm::defs::page::*;
use crate::defs::errno::Errno;
use crate::defs::macros::*;
use crate::defs::mm::*;
use crate::log_warn;
use crate::sync;
use core::arch::asm;
use core::arch::global_asm;
use core::arch::size_of;

global_asm!(include_str!("lib/memset.S"));
unsafe extern "C" {
     fn memset(src: u64, value: u32, len: u64) -> u64;
}

static spin: sync::SpinLock = sync::SpinLock::new();

pub fn pt_mapping_level(vfn: Vfn, mfn: Mfn, nr: u64, flags: Flags) -> usize {
    let mut mask = if !mfn_eq(mfn, INVALID_MFN) { mfn } else { 0 };
    mask |= vfn;

    return if !flags.contains(_PAGE_BLOCK) {
        3
    } else if is_lvl_one(mask, nr) {
        1
    } else if is_lvl_two(mask, nr) {
        2
    } else {
        3
    };
}

pub fn pt_check_contig(vfn: Vfn, mfn: Mfn, level: u32, left: u64, flags: Flags) -> usize {
    if !flags.contains(_PAGE_BLOCK) {
        return 1;
    }

    if mfn_eq(mfn, INVALID_MFN) {
        return 1;
    }

    const PT_4K_NR_CONTIG: usize = 16;
    let nr_contig = BIT(PT_LEVEL_ORDER(level as u64)) * 16;

    if left < nr_contig || ((mfn | vfn) & (nr_contig - 1)) > 0 {
        return 1;
    }

    return PT_4K_NR_CONTIG;
}

struct TableGuard {
    ptr: *mut Pte,
}

impl TableGuard {
    pub fn new(root: Mfn) -> Self {
        let ptr = map_table(root);
        Self { ptr }
    }

    pub fn entry(&mut self, offset: usize) -> &mut Pte {
        unsafe { &mut *self.ptr.add(offset) }
    }

    pub fn remap(&mut self, mfn: Mfn) {
        unmap_table(self.ptr);
        self.ptr = map_table(mfn);
    }
}

impl Drop for TableGuard {
    fn drop(&mut self) {
        unmap_table(self.ptr);
    }
}

const TABLE_SUPER_PAGE: u8 = 1;
const TABLE_NORMAL_PAGE: u8 = 2;

fn pt_next_level(
    im_not_mapping_that: bool,
    level: usize,
    table: &mut TableGuard,
    offset: usize,
) -> Result<u8, Errno> {
    let entry = table.entry(offset);

    if !entry.is_valid() {
        if im_not_mapping_that {
            return Err(Errno::HEMITHEOS_TABLE_MAP_FAILED);
        }

        let ret = create_table();
        if (ret != 0) {
            return Err(Errno::HEMITHEOS_TABLE_MAP_FAILED);
        }
    }

    debug_assert!(level < 3);

    if entry.kind(level as u8) == PteKind::SuperBlock {
        return Ok(TABLE_SUPER_PAGE);
    }

    let mfn: Mfn = entry.mfn();

    table.remap(mfn);

    Ok(TABLE_NORMAL_PAGE)
}

pub fn pt_update_entry(
    root: Mfn,
    virt: VAddr,
    mfn: Mfn,
    target: u64,
    flags: Flags,
) -> Result<u8, Errno> {
    /*
     * For missing tables below lvl.3: Aborts if both the MFN and populate bit
     * are invalid. However, if the MFN is valid, it forces auto-population
     * regardless of the populate bit, freeing the caller from intermediate table management.
     */
    let level;
    let local_pte: Pte;
    let im_not_mapping_that = mfn_eq(mfn, INVALID_MFN) && !flags.contains(_PAGE_POPULATE);

    let offsets = EXTRACT_OFFSETS_FROM_ADDR(virt);

    assert!(flags.contains(_PAGE_POPULATE | _PAGE_PRESENT) != true);

    let table: &mut TableGuard = map_table(root);

    let mut rc: u8 = 0;

    for level in (HYP_PT_ROOT_LEVEL as usize)..(target as usize) {
        rc = match pt_next_level(im_not_mapping_that, level, table, offsets[level]) {
            Ok(value) => value,
            Err(err) => {
                if flags.intersects(_PAGE_PRESENT | _PAGE_PRESENT) {
                    return Err(err);
                }

                return Ok(0);
            }
        };
        if rc != TABLE_NORMAL_PAGE {
            break;
        }
    }

    if level != target {
        return Err(Errno::HEMITHEOS_TABLE_MAP_FAILED);
    }

    let entry = table.entry(level);

    if (pt_check_entry(entry, mfn, level, flags) == false) {
        return Err(Errno::HEMITHEOS_EINVAL);
    }

    if flags & _PAGE_POPULATE == 1 {
        return Ok(0)
    }

    if flags & _PAGE_PRESENT == 0 {
        unsafe { memset(local_pte.raw, 0x00, size_of(Pte)); }
    }

    Ok(0)
}

pub fn pt_update(virt: VAddr, mfn: Mfn, nr_mfns: u64, flags: Flags) -> Result<u8, Errno> {
    let vfn: Vfn = get_afn(virt);
    let mut left: u64 = nr_mfns;
    let root_raw: u64;

    unsafe {
        asm!(
            "mrs {}, TTBR0_EL2",
            out(reg) root_raw
        );
    }

    let root: Mfn = root_raw;

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

    spin.lock();

    while left > 0 {
        let level = pt_mapping_level(vfn, mfn, left, flags);
        let order = PT_LEVEL_ORDER(level as u64);

        assert!(left >= BIT(order));

        let nr_contig = pt_check_contig(vfn, mfn, level as u32, left, flags);
        let new_flags = flags | if nr_contig > 1 { _PAGE_CONTIG } else { 0 };

        /*for _idx in 0..nr_contig {
            /*rc = pt_update_entry();*/
        }*/
    }

    spin.unlock();

    Ok(0)
}
