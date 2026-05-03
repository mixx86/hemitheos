#![no_std]
#![no_main]

use hemitheos::dtb;
use hemitheos::uart;

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn main(dtb_addr: usize) -> ! {
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    uart::puts("PANIC!\n");

    loop {
        unsafe {
            #[cfg(target_arch = "aarch64")]
            core::arch::asm!("wfe", options(nostack, nomem));
            #[cfg(target_arch = "riscv64")]
            core::arch::asm!("wfi", options(nostack, nomem));
        }
    }
}
