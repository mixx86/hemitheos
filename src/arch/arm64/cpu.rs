use crate::boot::BootSource;
use crate::platform::MAX_SMP_CPUS;
use crate::sync::Once;

// Strips RES1 (bit 31), U (bit 30), MT (bit 24), and all RES0 bits, leaving only
// Aff3[39:32], Aff2[23:16], Aff1[15:8], Aff0[7:0] — the fields that uniquely
// identify a core across all topologies including multi-cluster servers.
const MPIDR_HWID_MASK: u64 = 0x00_00FF_00FF_FFFF;

struct CpuTable {
    mpidrs: [u64; MAX_SMP_CPUS],
    count: usize,
}

static CPU_TABLE: Once<CpuTable> = Once::new();

fn try_probe_from_fdt(fdt: &fdt::Fdt) -> Option<()> {
    let mut mpidrs = [0u64; MAX_SMP_CPUS];
    let mut count = 0;

    for (i, cpu) in fdt.cpus().enumerate().take(MAX_SMP_CPUS) {
        mpidrs[i] = (cpu.ids().first() as u64) & MPIDR_HWID_MASK;
        count += 1;
    }

    if count == 0 {
        return None;
    }

    CPU_TABLE.set(CpuTable { mpidrs, count });
    Some(())
}

// At minimum the running core is always known, even with no external topology source.
fn try_probe_from_mpidr() -> bool {
    let mut mpidrs = [0u64; MAX_SMP_CPUS];
    mpidrs[0] = current_cpu_mpidr();
    CPU_TABLE.set(CpuTable { mpidrs, count: 1 });
    true
}

fn fallback_single_cpu() {
    CPU_TABLE.set(CpuTable {
        mpidrs: [0u64; MAX_SMP_CPUS],
        count: 1,
    });
}

fn current_cpu_mpidr() -> u64 {
    let mpidr: u64;
    // SAFETY: MPIDR_EL1 is always readable from EL2; read-only with no side effects.
    unsafe {
        core::arch::asm!("mrs {}, mpidr_el1", out(reg) mpidr, options(nostack, nomem));
    }
    mpidr & MPIDR_HWID_MASK
}

pub fn probe_cpus(source: &BootSource) {
    let found = crate::boot::probe(
        source,
        |fdt| try_probe_from_fdt(fdt),
        || todo!("ACPI MADT CPU discovery"),
        || todo!("proprietary firmware CPU discovery"),
    );

    if found.is_none() {
        if !try_probe_from_mpidr() {
            fallback_single_cpu();
        }
    }
}

pub fn cpu_count() -> usize {
    CPU_TABLE.get().map(|t| t.count).unwrap_or(0)
}

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
