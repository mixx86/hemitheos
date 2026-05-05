use core::prelude::v1::*;

use crate::sync;

static PLATFORM: sync::Once<PlatformInfo> = sync::Once::new();

pub struct PlatformInfo {
    /// Discovered console (base address, driver type, input clock).
    pub console: Option<crate::console::ConsoleInfo>,
    /// GICv3 distributor base address.
    pub gicd_base: u64,
    /// GICv3 redistributor base address (first redistributor frame).
    pub gicr_base: u64,
    /// GICv3 redistributor region size in bytes (all redistributor frames combined).
    pub gicr_size: u64,
    /// Number of CPUs discovered from the /cpus DTB node.
    pub num_cpus: usize,
    /// Physical base address of the first RAM region.
    pub ram_base: u64,
    /// Total RAM size in bytes (sum of all /memory regions).
    pub ram_size: u64,
}

fn validate_dtb_address(addr: usize) -> bool {
    if addr == 0 {
        return false;
    }
    let magic = unsafe { core::ptr::read_volatile(addr as *const u32) };
    u32::from_be(magic) == 0xD00D_FEED
}

pub fn parse_dtb(dtb_addr: usize) -> PlatformInfo {
    if dtb_addr == 0 {
        panic!("DTB address is null — check ARM64 Image header and boot.s");
    }

    if !validate_dtb_address(dtb_addr) {
        panic!("Invalid DTB address");
    }

    // SAFETY: addr is non-null and readable (magic confirmed by validate_dtb_address);
    // the fdt crate validates the DTB header length and structure internally.
    let fdt = unsafe { fdt::Fdt::from_ptr(dtb_addr as *const u8).expect("Failed to parse DTB") };

    let mut platform = PlatformInfo {
        console: None,
        gicd_base: 0,
        gicr_base: 0,
        gicr_size: 0,
        num_cpus: 0,
        ram_base: 0,
        ram_size: 0,
    };

    let memory = fdt.memory();
    let mut regions = memory.regions().peekable();
    if regions.peek().is_none() {
        panic!("No memory regions in DTB");
    }

    for region in regions {
        let base = region.starting_address as u64;
        let size = region.size.unwrap_or(0) as u64;

        if platform.ram_base == 0 {
            platform.ram_base = base;
        }

        platform.ram_size += size;
    }

    platform.console = crate::console::find_console(&fdt);

    let cpu_count = fdt.cpus().count();
    if cpu_count == 0 {
        panic!("No CPUs found in DTB");
    }

    platform.num_cpus = cpu_count;

    let gic = fdt
        .find_compatible(&["arm,gic-v3"])
        .expect("GICv3 not found in DTB");

    let mut regs = gic.reg().expect("GIC reg missing");

    let gicd = regs.next().expect("missing GICD region");

    platform.gicd_base = gicd.starting_address as u64;

    let gicr = regs.next().expect("missing GICR region");

    platform.gicr_base = gicr.starting_address as u64;
    platform.gicr_size = gicr.size.unwrap_or(0) as u64;

    platform
}

pub fn init(dtb_addr: usize) {
    PLATFORM.set(parse_dtb(dtb_addr));
}

pub fn platform() -> &'static PlatformInfo {
    PLATFORM
        .get()
        .expect("platform not initialized — call platform::init first")
}
