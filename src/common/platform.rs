use crate::boot::BootSource;
use crate::sync;

static PLATFORM: sync::Once<PlatformInfo> = sync::Once::new();

pub const MAX_SMP_CPUS: usize = 8;


pub struct PlatformInfo {
    pub console: Option<crate::console::ConsoleInfo>,

    pub num_cpus: usize,

    pub ram_base: u64,
    pub ram_size: u64,
}

fn discover_cpus(source: &BootSource) -> usize {
    crate::arch::probe_cpus(source);
    let count = crate::arch::cpu_count();

    if count > 0 {
        count
    } else if let BootSource::Fdt(fdt) = source {
        fdt.cpus().count().min(MAX_SMP_CPUS)
    } else {
        0
    }
}

fn try_init_from_dtb(boot_arg: usize) -> Option<PlatformInfo> {
    if boot_arg == 0 {
        return None;
    }

    // SAFETY: boot_arg is non-null; the firmware contract guarantees it points
    // to readable memory. We read 4 bytes to check the FDT magic before going further.
    let magic = unsafe { core::ptr::read_volatile(boot_arg as *const u32) };
    if u32::from_be(magic) != 0xD00D_FEED {
        return None;
    }

    // SAFETY: magic confirmed above; fdt validates the header length and structure.
    let fdt = unsafe { fdt::Fdt::from_ptr(boot_arg as *const u8).ok()? };

    let mut platform = PlatformInfo {
        console: None,
        num_cpus: 0,
        ram_base: 0,
        ram_size: 0,
    };

    let memory = fdt.memory();
    let mut regions = memory.regions().peekable();
    regions.peek()?;

    for region in regions {
        let base = region.starting_address as u64;
        let size = region.size.unwrap_or(0) as u64;

        if platform.ram_base == 0 {
            platform.ram_base = base;
        }

        platform.ram_size += size;
    }

    platform.console = crate::console::find_console(&fdt);

    platform.num_cpus = discover_cpus(&BootSource::Fdt(&fdt));

    Some(platform)
}

fn try_init_from_acpi() -> Option<PlatformInfo> {
    // TODO: ACPI platform discovery (MADT, SRAT, GTDT, ...)
    None
}

fn try_init_from_firmware() -> Option<PlatformInfo> {
    // TODO: proprietary firmware platform discovery
    None
}

pub fn init(boot_arg: usize) {
    let platform = try_init_from_dtb(boot_arg)
        .or_else(try_init_from_acpi)
        .or_else(try_init_from_firmware)
        .expect("no platform information source available");

    PLATFORM.set(platform);
}

pub(crate) fn try_platform() -> Option<&'static PlatformInfo> {
    PLATFORM.get()
}

pub fn platform() -> &'static PlatformInfo {
    try_platform().expect("platform not initialized — call platform::init first")
}
