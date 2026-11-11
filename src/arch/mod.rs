pub mod arm;

pub use arm::*;

use crate::{arch::mmu::mmu::ioremap, defs::errno::Errno};

pub fn arch_ioremap(paddr: u64, size: u64) -> Result<u64, Errno> {
    ioremap(paddr, size)
}

pub fn early_setup(boot_arg: usize) {
    setup::early_setup(boot_arg);
}

// Stubs for host compilation (tests on dev machine).
#[cfg(not(has_arch_impl))]
pub fn current_cpu_index() -> usize {
    0
}

#[cfg(not(has_arch_impl))]
pub fn probe_cpus(_source: &crate::boot::BootSource) {}

#[cfg(not(has_arch_impl))]
pub fn cpu_count() -> usize {
    0
}
