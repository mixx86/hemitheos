const VIRT_DTB: &[u8] = include_bytes!("fixtures/qemu-virt-aarch64.dtb");

// parse_dtb reads the first word via read_volatile(*const u32), requiring 4-byte alignment.
// Vec<u32> guarantees this. The bytes are copied without reinterpreting their values.
fn aligned_copy(src: &[u8]) -> Vec<u32> {
    let mut buf = vec![0u32; (src.len() + 3) / 4];
    let bytes = unsafe { std::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, src.len()) };
    bytes.copy_from_slice(src);
    buf
}

#[test]
fn find_console_does_not_panic_on_valid_dtb() {
    let fdt = fdt::Fdt::new(VIRT_DTB).expect("virt.dtb must be a valid FDT");
    let result = hemitheos::console::find_console(&fdt);

    if let Some(info) = result {
        // A MMIO peripheral at address zero is not valid on any known platform.
        assert_ne!(info.base, 0, "console base must be non-zero");
        // find_console enforces this range internally before returning Some.
        assert!(
            (1_000_000..=500_000_000).contains(&info.clock_hz),
            "clock_hz {} is outside the 1 MHz–500 MHz range accepted by the driver",
            info.clock_hz
        );
    }
    // None is valid: not all DTBs expose a console via /chosen, /aliases, or compatible scan.
}

#[test]
fn parse_dtb_with_real_dtb_returns_sane_platform() {
    let buf = aligned_copy(VIRT_DTB);
    let platform = hemitheos::platform::parse_dtb(buf.as_ptr() as usize);

    // parse_dtb panics if either of these is zero, so the assertions also document
    // what the function already guarantees.
    assert!(platform.num_cpus > 0, "platform must have at least one CPU");
    assert!(platform.ram_size > 0, "platform must have RAM");

    // GIC base addresses: address zero is not a valid MMIO mapping on any ARMv8 platform.
    assert_ne!(platform.gicd_base, 0, "GICD base must be non-zero");
    assert_ne!(platform.gicr_base, 0, "GICR base must be non-zero");

    // GICD and GICR are physically separate register banks and must not alias.
    assert_ne!(
        platform.gicd_base, platform.gicr_base,
        "GICD and GICR must be at distinct addresses"
    );
}
