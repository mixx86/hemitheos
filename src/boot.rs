// Describes the source of boot information. Platform parses and classifies the
// data; arch receives the typed source and operates on it directly.
pub enum BootSource<'a> {
    Fdt(&'a fdt::Fdt<'a>),
    Acpi,     // TODO: add ACPI table type when implementing
    Firmware, // TODO: add firmware-specific type when implementing
}

// Dispatches to the appropriate closure based on the boot source variant.
// Centralises the source-selection logic so callers only write the per-source
// behaviour, not the dispatch.
pub fn probe<'a, T>(
    source: &'a BootSource<'a>,
    from_dtb: impl FnOnce(&'a fdt::Fdt<'a>) -> Option<T>,
    from_acpi: impl FnOnce() -> Option<T>,
    from_firmware: impl FnOnce() -> Option<T>,
) -> Option<T> {
    match source {
        BootSource::Fdt(fdt) => from_dtb(fdt),
        BootSource::Acpi => from_acpi(),
        BootSource::Firmware => from_firmware(),
    }
}
