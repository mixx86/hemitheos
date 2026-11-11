use crate::{
    arch::defs::{BOOT_INFO_VIRT_START, PAGE_SIZE},
    arch::mmu::pt::virt_to_maddr,
    arch::mmu::setup::{early_boot_info_map, setup_directmap_mappings},
    common::page_alloc,
    defs::{
        macros::MB,
        mm::{MAddr, VAddr},
        system::{SysState, set_system_state},
    },
    platform::{self, PlatformInfo},
};

unsafe extern "C" {
    static _end: u8;
}

pub fn early_setup(boot_arg: usize)
{
    early_boot_info_map(boot_arg as MAddr);

    let fdt_vaddr = BOOT_INFO_VIRT_START + (boot_arg as u64 & (PAGE_SIZE - 1));
    platform::init(fdt_vaddr as usize);

    setup_mm(platform::platform(), boot_arg as MAddr);
    set_system_state(SysState::SYS_STATE_boot);
}

fn setup_mm(info: &PlatformInfo, boot_info_paddr: MAddr)
{
    setup_directmap_mappings(info.ram_base, info.ram_size);

    let image_end = match virt_to_maddr(unsafe { &raw const _end } as VAddr - 1) {
        Ok(paddr) => paddr + 1,
        Err(errno) => panic!("setup_mm: cannot translate _end: {:?}", errno),
    };

    page_alloc::init(
        image_end,
        info.ram_base + info.ram_size,
        boot_info_paddr,
        boot_info_paddr + MB(2),
    );
}
