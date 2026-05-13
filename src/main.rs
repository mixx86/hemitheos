#![no_std]
#![no_main]

use hemitheos::{console, log_error, log_info, platform};

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn main(boot_arg: usize) -> ! {
    platform::init(boot_arg);
    console::init(platform::platform().console);
    log_info!("Hemitheos\n");

    loop {
        unsafe {
            #[cfg(target_arch = "aarch64")]
            core::arch::asm!("wfe", options(nostack, nomem));
            #[cfg(target_arch = "riscv64")]
            core::arch::asm!("wfi", options(nostack, nomem));
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log_error!("Panic! :(\n");
    log_error!("{}", info);

    loop {
        unsafe {
            #[cfg(target_arch = "aarch64")]
            core::arch::asm!("wfe", options(nostack, nomem));
            #[cfg(target_arch = "riscv64")]
            core::arch::asm!("wfi", options(nostack, nomem));
        }
    }
}
