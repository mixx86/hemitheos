const UARTCR: u64 = 0x30;
const UARTIBRD: u64 = 0x24;
const UARTFBRD: u64 = 0x28;
const UARTLCR_H: u64 = 0x2c;
const UARTFR: u64 = 0x18;

const UARTCR_UARTEN: u32 = 1 << 0;
const UARTCR_TXE: u32 = 1 << 8;
const UARTLCR_H_FEN: u32 = 1 << 4;
const UARTLCR_H_WLEN_8: u32 = 0b11 << 5;

pub struct Pl011 {
    base: u64,
    clock_hz: u32,
}

impl Pl011 {
    pub fn new(base: u64, clock_hz: u32) -> Self {
        Self { base, clock_hz }
    }

    pub fn init(&self) {
        // Sequence follows the PL011 TRM initialisation procedure:
        // disable → configure baud/format → enable.
        self.write_reg(UARTCR, 0x00);

        let baud = 115200u32;
        let divisor = self.clock_hz / (16 * baud);
        let int_div = divisor;
        let frac_div = (((self.clock_hz % (16 * baud)) * 64) + (8 * baud)) / (16 * baud);
        self.write_reg(UARTIBRD, int_div);
        self.write_reg(UARTFBRD, frac_div);

        self.write_reg(UARTLCR_H, UARTLCR_H_WLEN_8 | UARTLCR_H_FEN);
        self.write_reg(UARTCR, UARTCR_UARTEN | UARTCR_TXE);
    }
    #[inline]
    fn read_reg(&self, offset: u64) -> u32 {
        // SAFETY: base is a valid 4-byte-aligned MMIO address; offset is a
        // module-constant register offset. Volatile prevents the compiler from
        // eliding or reordering the read.
        unsafe { core::ptr::read_volatile((self.base + offset) as *const u32) }
    }

    #[inline]
    fn write_reg(&self, offset: u64, val: u32) {
        // SAFETY: same aliasing and alignment guarantees as read_reg. Volatile
        // ensures the write reaches the hardware and is not reordered or elided.
        unsafe { core::ptr::write_volatile((self.base + offset) as *mut u32, val) }
    }
}

impl crate::console::ConsoleDriver for Pl011 {
    fn puts(&self, s: &str) {
        for &b in s.as_bytes() {
            // Polling TXFF (bit 5) before each write prevents overflowing the 16-byte TX FIFO.
            while self.read_reg(UARTFR) & (1 << 5) != 0 {}
            self.write_reg(0x00, b as u32);
        }
        // Polling BUSY (bit 3) ensures all bytes have been physically transmitted
        // before returning, preventing data loss if the caller resets or disables the UART.
        while self.read_reg(UARTFR) & (1 << 3) != 0 {}
    }
}

static DRIVER: crate::sync::Once<Pl011> = crate::sync::Once::new();

pub fn init(base: u64, clock_hz: u32) -> &'static dyn crate::console::ConsoleDriver {
    let pl011 = Pl011::new(base, clock_hz);
    pl011.init();
    DRIVER.set(pl011);
    DRIVER.get().unwrap()
}
