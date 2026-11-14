use crate::{
    arch::defs::{BOOT_INFO_VIRT_START, PAGE_SIZE},
    arch::mmu::pt::virt_to_maddr,
    arch::mmu::setup::{early_boot_info_map, setup_directmap_mappings},
    common::page_alloc,
    defs::{
        macros::MB,
        mm::*,
        system::{SysState, set_system_state},
    },
    platform::{self, PlatformInfo},
};

unsafe extern "C" {
    static _start: u8;
    static _end: u8;
}

pub fn early_setup(boot_arg: usize)
{
    early_boot_info_map(boot_arg as MAddr);

    let boot_info_vaddr = BOOT_INFO_VIRT_START + (boot_arg as u64 & (PAGE_SIZE - 1));
    platform::init(boot_info_vaddr as usize);

    setup_mm(platform::platform(), boot_arg as MAddr);
    set_system_state(SysState::SYS_STATE_boot);
}

fn setup_mm(info: &PlatformInfo, boot_info_paddr: MAddr)
{
    let mut mappable_regions: MemRegions<MAX_MEM_REGIONS> = MemRegions::new();
    for bank in info.ram_banks.as_slice() {
        add_free_regions(&mut mappable_regions, bank, &info.no_map.as_slice());
    }
    for r in mappable_regions.as_slice() {
        let start = (r.start + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
        let end = r.end & !(PAGE_SIZE - 1);
        if start < end {
            setup_directmap_mappings(start, end - start);
        }
    }

    let image_start = image_paddr(unsafe { &raw const _start } as VAddr);
    let image_end = image_paddr(unsafe { &raw const _end } as VAddr - 1) + 1;

    let mut reserved: MemRegions<{ MAX_RESERVED + 2 }> = MemRegions::new();
    for r in info.reserved.as_slice() {
        reserved.add(r.start, r.end);
    }
    reserved.add(image_start, image_end);
    reserved.add(boot_info_paddr, boot_info_paddr + MB(2));

    let mut free: MemRegions<MAX_MEM_REGIONS> = MemRegions::new();
    for bank in info.ram_banks.as_slice() {
        add_free_regions(&mut free, bank, &reserved.as_slice());
    }

    page_alloc::init(free.as_slice().iter());
}

fn image_paddr(va: VAddr) -> MAddr
{
    match virt_to_maddr(va) {
        Ok(paddr) => paddr,
        Err(errno) => panic!("setup_mm: cannot translate image address: {:?}", errno),
    }
}

fn add_free_regions<const N: usize>(free: &mut MemRegions<N>, bank: &MemRegion, reserved: &[MemRegion])
{
    let mut cur = bank.start;

    while cur < bank.end {
        let next = reserved
            .iter()
            .filter(|r| r.end > cur && r.start < bank.end)
            .min_by_key(|r| r.start);

        match next {
            Some(r) => {
                free.add(cur, r.start);
                cur = r.end;
            }
            None => {
                free.add(cur, bank.end);
                break;
            }
        }
    }
}
