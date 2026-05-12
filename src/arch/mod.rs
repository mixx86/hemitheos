#[cfg(target_arch = "aarch64")]
pub mod arm64;
#[cfg(target_arch = "aarch64")]
pub use arm64::*;

// Stubs for host compilation (tests on dev machine).
// Only active when no real arch implementation exists (has_arch_impl not set).
#[cfg(not(has_arch_impl))]
pub fn current_cpu_index() -> usize { 0 }
#[cfg(not(has_arch_impl))]
pub fn probe_cpus(_source: &crate::boot::BootSource) {}
#[cfg(not(has_arch_impl))]
pub fn cpu_count() -> usize { 0 }
