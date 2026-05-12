use crate::boot::BootSource;
use crate::platform::MAX_SMP_CPUS;
use crate::sync::Once;

// Aff3[39:32] + Aff2[23:16] + Aff1[15:8] + Aff0[7:0]. Strips RES1, U, MT, and RES0.
const MPIDR_HWID_MASK: u64 = 0x00_00FF_00FF_FFFF;

struct CpuTable {
    mpidrs: [u64; MAX_SMP_CPUS],
    count: usize,
}

static CPU_TABLE: Once<CpuTable> = Once::new();

fn try_probe_from_dtb(source: &BootSource) -> bool {
    let fdt = match source {
        BootSource::Fdt(f) => f,
        _ => return false,
    };

    let mut mpidrs = [0u64; MAX_SMP_CPUS];
    let mut count = 0;

    for (i, cpu) in fdt.cpus().enumerate().take(MAX_SMP_CPUS) {
        mpidrs[i] = (cpu.ids().first() as u64) & MPIDR_HWID_MASK;
        count += 1;
    }

    if count == 0 {
        return false;
    }

    CPU_TABLE.set(CpuTable { mpidrs, count });
    true
}

fn try_probe_from_acpi(source: &BootSource) -> bool {
    match source {
        BootSource::Acpi => todo!("ACPI MADT CPU discovery"),
        _ => false,
    }
}

fn try_probe_from_firmware(source: &BootSource) -> bool {
    match source {
        BootSource::Firmware => todo!("proprietary firmware CPU discovery"),
        _ => false,
    }
}

// Guaranteed fallback: at minimum the running core is known.
fn try_probe_from_mpidr() -> bool {
    let mut mpidrs = [0u64; MAX_SMP_CPUS];
    mpidrs[0] = current_cpu_mpidr();
    CPU_TABLE.set(CpuTable { mpidrs, count: 1 });
    true
}

fn fallback_single_cpu() {
    CPU_TABLE.set(CpuTable { mpidrs: [0u64; MAX_SMP_CPUS], count: 1 });
}

fn current_cpu_mpidr() -> u64 {
    let mpidr: u64;
    unsafe {
        core::arch::asm!("mrs {}, mpidr_el1", out(reg) mpidr, options(nostack, nomem));
    }
    mpidr & MPIDR_HWID_MASK
}

// Discovers available CPUs trying each source in order until one succeeds.
pub fn probe_cpus(source: &BootSource) {
    if try_probe_from_dtb(source)      { return; }
    if try_probe_from_acpi(source)     { return; }
    if try_probe_from_firmware(source) { return; }
    if try_probe_from_mpidr()          { return; }
    fallback_single_cpu();
}

pub fn cpu_count() -> usize {
    CPU_TABLE.get().map(|t| t.count).unwrap_or(0)
}

// Returns the logical index (0..num_cpus) of the calling CPU for use as an array index.
//
// Before probe_cpus runs only core 0 is active, so falling back to Aff0 is safe.
pub fn current_cpu_index() -> usize {
    let mpidr = current_cpu_mpidr();
    match CPU_TABLE.get() {
        Some(t) => t.mpidrs[..t.count]
            .iter()
            .position(|&m| m == mpidr)
            .unwrap_or(0),
        None => (mpidr & 0xFF) as usize,
    }
}
