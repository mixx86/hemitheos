use crate::{
    arch::defs::{BOOT_INFO_VIRT_START, PAGE_SIZE},
    arch::mmu::pt::virt_to_maddr,
    arch::mmu::setup::{early_boot_info_map, setup_directmap_mappings},
    common::page_alloc,
    defs::{
        macros::MB,
        mm::{MAddr, MemRegion, MemRegions, VAddr},
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
    setup_directmap_mappings(info.ram_base, info.ram_size);

    let image_start = image_paddr(unsafe { &raw const _start } as VAddr);
    let image_end = image_paddr(unsafe { &raw const _end } as VAddr - 1) + 1;

    let mut reserved = MemRegions::new();
    for r in info.reserved.as_slice() {
        reserved.add(r.start, r.end);
    }
    reserved.add(image_start, image_end);
    reserved.add(boot_info_paddr, boot_info_paddr + MB(2));

    let mut free = MemRegions::new();
    for bank in info.ram_banks.as_slice() {
        add_free_regions(&mut free, bank, &reserved);
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

fn add_free_regions(free: &mut MemRegions, bank: &MemRegion, reserved: &MemRegions)
{
    let mut cur = bank.start;

    while cur < bank.end {
        let next = reserved
            .as_slice()
            .iter()
            .filter(|r| r.end > cur && r.start < bank.end)
            .min_by_key(|r| r.start);

        match next {
            Some(r) => {
                free.add(cur, r.start);
                cur = cur.max(r.end);
            }
            None => {
                free.add(cur, bank.end);
                break;
            }
        }
    }
}
