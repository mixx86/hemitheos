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
    pub baud_rate: u32, // 0 = use driver default (115200)
}

pub fn init(info: Option<ConsoleInfo>) {
    let info = match info {
        Some(info) => info,
        None => return,
    };
    let driver: &'static dyn ConsoleDriver = match info.kind {
        ConsoleType::Pl011 => {
            crate::drivers::uart::pl011::init(info.base, info.clock_hz, info.baud_rate)
        }
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
        .expect("console not initialized — call console::init first")
}

pub fn puts(s: &str) {
    if let Some(driver) = CONSOLE.get().copied() {
        driver.puts(s);
    }
}

// ── Console discovery ─────────────────────────────────────────────────────────

fn read_be_u32(data: &[u8]) -> Option<u32> {
    let bytes: [u8; 4] = data.get(0..4)?.try_into().ok()?;
    Some(u32::from_be_bytes(bytes))
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

fn extract_console_clock(fdt: &fdt::Fdt, node: &fdt::node::FdtNode) -> Option<u32> {
    const CLOCK_PROPS: &[&str] = &["clock-frequency", "uart-clock-frequency", "clock-hz"];

    for prop_name in CLOCK_PROPS {
        if let Some(prop) = node.property(prop_name)
            && let Some(value) = read_be_u32(prop.value)
            && (1_000_000..=500_000_000).contains(&value)
        {
            return Some(value);
        }
    }

    // Many platforms (including QEMU virt) define the clock via a phandle in
    // 'clocks' rather than a direct property on the peripheral node.
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

fn find_console_in_fdt(fdt: &fdt::Fdt) -> Option<ConsoleInfo> {
    let node_to_console_info = |node: fdt::node::FdtNode| -> Option<ConsoleInfo> {
        let reg = node.reg()?.next()?;
        let mut kind = ConsoleType::Unknown;

        if let Some(compat) = node.compatible()
            && let Some(compat_str) = compat.all().next()
        {
            kind = console_type_from_compatible(compat_str);
        }

        let clock_hz = extract_console_clock(fdt, &node)?;
        let baud_rate = node
            .property("current-speed")
            .and_then(|p| read_be_u32(p.value))
            .unwrap_or(0);

        Some(ConsoleInfo {
            kind,
            base: reg.starting_address as u64,
            clock_hz,
            baud_rate,
        })
    };

    if let Some(chosen) = fdt.find_node("/chosen") {
        for prop_name in ["stdout-path", "stdout", "linux,stdout-path"] {
            if let Some(prop) = chosen.property(prop_name)
                && let Ok(path) = core::str::from_utf8(prop.value)
            {
                // Strip null terminator and Linux console options (e.g. ":115200n8").
                let node_path = path.trim_end_matches('\0').split(':').next().unwrap_or("");
                if let Some(node) = fdt.find_node(node_path)
                    && let Some(info) = node_to_console_info(node)
                {
                    return Some(info);
                }
            }
        }
    }

    if let Some(aliases) = fdt.find_node("/aliases") {
        for alias in ["serial0", "uart0", "console"] {
            if let Some(prop) = aliases.property(alias)
                && let Ok(path) = core::str::from_utf8(prop.value)
            {
                let node_path = path.trim_end_matches('\0').split(':').next().unwrap_or("");
                if let Some(node) = fdt.find_node(node_path)
                    && let Some(info) = node_to_console_info(node)
                {
                    return Some(info);
                }
            }
        }
    }

    for node in fdt.all_nodes() {
        if let Some(compat) = node.compatible() {
            for compat_str in compat.all() {
                let t = console_type_from_compatible(compat_str);
                if t != ConsoleType::Unknown
                    && let Some(info) = node_to_console_info(node)
                {
                    return Some(info);
                }
            }
        }
    }

    None
}

pub fn find_console(source: &crate::boot::BootSource) -> Option<ConsoleInfo> {
    crate::boot::probe(
        source,
        find_console_in_fdt,
        || None, // TODO: ACPI console discovery (SPCR table)
        || None, // TODO: proprietary firmware console discovery
    )
}
