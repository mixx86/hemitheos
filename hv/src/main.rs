#![no_std]
#![no_main]

mod arch;

use hemitheos::console;
use hemitheos::platform;

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn main(dtb_addr: usize) -> ! {
    let platform = platform::parse_dtb(dtb_addr);
    console::init(platform.console);
    console::puts("hey");
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
fn panic(_info: &PanicInfo) -> ! {
    console::puts("PANIC!\n");

    loop {
        unsafe {
            #[cfg(target_arch = "aarch64")]
            core::arch::asm!("wfe", options(nostack, nomem));
            #[cfg(target_arch = "riscv64")]
            core::arch::asm!("wfi", options(nostack, nomem));
        }
    }
}
