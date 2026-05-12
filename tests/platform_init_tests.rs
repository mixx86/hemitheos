const SAMPLE_FDT: &[u8] = include_bytes!("fixtures/qemu-virt-aarch64.dtb");

// platform::init reads the first word via read_volatile(*const u32), requiring 4-byte alignment.
// Vec<u32> guarantees this. The bytes are copied without reinterpreting their values.
fn aligned_copy(src: &[u8]) -> Vec<u32> {
    let mut buf = vec![0u32; (src.len() + 3) / 4];
    let bytes = unsafe { std::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut u8, src.len()) };
    bytes.copy_from_slice(src);
    buf
}

#[test]
fn find_console_does_not_panic_on_valid_fdt() {
    let fdt = fdt::Fdt::new(SAMPLE_FDT).expect("sample fdt must be valid");
    let result = hemitheos::console::find_console(&fdt);

    if let Some(info) = result {
        assert_ne!(info.base, 0, "console base must be non-zero");
        assert!(
            (1_000_000..=500_000_000).contains(&info.clock_hz),
            "clock_hz {} is outside the 1 MHz–500 MHz range accepted by the driver",
            info.clock_hz
        );
    }
}

#[test]
fn init_with_valid_fdt_returns_sane_platform() {
    let buf = aligned_copy(SAMPLE_FDT);
    hemitheos::platform::init(buf.as_ptr() as usize);
    let p = hemitheos::platform::platform();

    assert!(p.num_cpus > 0, "platform must have at least one CPU");
    assert!(p.ram_size > 0, "platform must have RAM");
}
