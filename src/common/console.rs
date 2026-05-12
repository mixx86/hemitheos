use crate::sync;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ConsoleType {
    Pl011,
    Ns16550,
    Msm,
    DwApb,
    Unknown,
}

pub trait ConsoleDriver: Sync {
    fn puts(&self, s: &str);
}

static CONSOLE: sync::Once<&'static dyn ConsoleDriver> = sync::Once::new();

#[derive(Copy, Clone)]
pub struct ConsoleInfo {
    pub kind: ConsoleType,
    pub base: u64,
    pub clock_hz: u32,
}

pub fn init(info: Option<ConsoleInfo>) {
    let info = match info {
        Some(info) => info,
        None => return,
    };
    let driver: &'static dyn ConsoleDriver = match info.kind {
        ConsoleType::Pl011 => crate::drivers::uart::pl011::init(info.base, info.clock_hz),
        ConsoleType::Ns16550 => return,
        ConsoleType::Msm => return,
        ConsoleType::DwApb => return,
        ConsoleType::Unknown => return,
    };

    CONSOLE.set(driver);
}

pub fn console() -> &'static dyn ConsoleDriver {
    CONSOLE
        .get()
        .copied()
        .expect("platform not initialized — call dtb::init first")
}

pub fn puts(s: &str) {
    if let Some(driver) = CONSOLE.get().copied() {
        driver.puts(s);
    }
}

fn console_type_from_compatible(compat: &str) -> ConsoleType {
    match compat {
        "arm,pl011" | "broadcom,pl011" => ConsoleType::Pl011,
        "ns16550a" | "ns16550" => ConsoleType::Ns16550,
        "snps,dw-apb-uart" => ConsoleType::DwApb,
        "qcom,msm-uartdm" => ConsoleType::Msm,
        _ => ConsoleType::Unknown,
    }
}

fn read_be_u32(data: &[u8]) -> Option<u32> {
    let bytes: [u8; 4] = data.get(0..4)?.try_into().ok()?;
    Some(u32::from_be_bytes(bytes))
}

fn extract_uart_clock(fdt: &fdt::Fdt, node: &fdt::node::FdtNode) -> Option<u32> {
    const CLOCK_PROPS: &[&str] = &[
        "clock-frequency",
        "current-speed",
        "uart-clock-frequency",
        "clock-hz",
    ];

    for prop_name in CLOCK_PROPS {
        let prop = match node.property(prop_name) {
            Some(p) => p,
            None => continue,
        };

        let value = match read_be_u32(prop.value) {
            Some(v) => v,
            None => continue,
        };

        if (1_000_000..=500_000_000).contains(&value) {
            return Some(value);
        }
    }

    // Many platforms (including QEMU virt) define the UART clock via a
    // phandle in the 'clocks' property rather than a direct 'clock-frequency'.
    // Walk the first phandle and read clock-frequency from the provider node.
    if let Some(clocks_prop) = node.property("clocks")
        && let Some(phandle) = read_be_u32(clocks_prop.value)
        && let Some(clock_node) = fdt.find_phandle(phandle)
        && let Some(freq_prop) = clock_node.property("clock-frequency")
        && let Some(hz) = read_be_u32(freq_prop.value)
        && (1_000_000..=500_000_000).contains(&hz)
    {
        return Some(hz);
    }

    None
}

pub fn find_console(fdt: &fdt::Fdt) -> Option<ConsoleInfo> {
    let node_to_uart_info = |node: fdt::node::FdtNode| -> Option<ConsoleInfo> {
        let reg = node.reg()?.next()?;

        let mut uart_type = ConsoleType::Unknown;

        if let Some(compat) = node.compatible()
            && let Some(compat_str) = compat.all().next()
        {
            uart_type = console_type_from_compatible(compat_str);
        }
        let clock_hz = extract_uart_clock(fdt, &node)?;
        Some(ConsoleInfo {
            kind: uart_type,
            base: reg.starting_address as u64,
            clock_hz,
        })
    };

    if let Some(chosen) = fdt.find_node("/chosen") {
        for prop_name in ["stdout-path", "stdout", "linux,stdout-path"] {
            if let Some(prop) = chosen.property(prop_name)
                && let Ok(path) = core::str::from_utf8(prop.value)
            {
                let node_path = path.trim_end_matches('\0');
                if let Some(node) = fdt.find_node(node_path)
                    && let Some(console) = node_to_uart_info(node)
                {
                    return Some(console);
                }
            }
        }
    }

    if let Some(aliases) = fdt.find_node("/aliases") {
        for alias in ["serial0", "uart0", "console"] {
            if let Some(prop) = aliases.property(alias)
                && let Ok(path) = core::str::from_utf8(prop.value)
            {
                let node_path = path.trim_end_matches('\0');
                if let Some(node) = fdt.find_node(node_path)
                    && let Some(uart) = node_to_uart_info(node)
                {
                    return Some(uart);
                }
            }
        }
    }

    for node in fdt.all_nodes() {
        if let Some(compat) = node.compatible() {
            for compat_str in compat.all() {
                let t = console_type_from_compatible(compat_str);
                if t != ConsoleType::Unknown
                    && let Some(uart) = node_to_uart_info(node)
                {
                    return Some(uart);
                }
            }
        }
    }

    None
}
