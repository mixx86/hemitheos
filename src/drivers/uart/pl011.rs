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
    baud: u32,
}

impl Pl011 {
    pub fn new(base: u64, clock_hz: u32, baud: u32) -> Self {
        Self { base, clock_hz, baud }
    }

    pub fn init(&self) {
        // PL011 TRM requires: disable → configure baud/format → enable.
        // Writing to IBRD/FBRD/LCR_H while enabled has undefined behaviour.
        self.write_reg(UARTCR, 0x00);

        let baud = if self.baud > 0 { self.baud } else { 115200 };
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
        // SAFETY: base is a valid MMIO address supplied by platform discovery;
        // offset is a module-constant within the PL011 register map.
        // Volatile prevents the compiler from eliding or reordering the access.
        unsafe { core::ptr::read_volatile((self.base + offset) as *const u32) }
    }

    #[inline]
    fn write_reg(&self, offset: u64, val: u32) {
        // SAFETY: same guarantees as read_reg.
        unsafe { core::ptr::write_volatile((self.base + offset) as *mut u32, val) }
    }
}

impl crate::console::ConsoleDriver for Pl011 {
    fn puts(&self, s: &str) {
        for &b in s.as_bytes() {
            // TXFF (bit 5): TX FIFO is full. Spin to avoid overflowing the 16-byte FIFO.
            while self.read_reg(UARTFR) & (1 << 5) != 0 {}
            self.write_reg(0x00, b as u32);
        }
        // BUSY (bit 3): transmission in progress. Drain before returning so the
        // caller can safely reset or disable the UART immediately after puts().
        while self.read_reg(UARTFR) & (1 << 3) != 0 {}
    }
}

static DRIVER: crate::sync::Once<Pl011> = crate::sync::Once::new();

pub fn init(base: u64, clock_hz: u32, baud_rate: u32) -> &'static dyn crate::console::ConsoleDriver {
    let pl011 = Pl011::new(base, clock_hz, baud_rate);
    pl011.init();
    DRIVER.set(pl011);
    DRIVER.get().unwrap()
}
