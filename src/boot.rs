// Describes the source of boot information. Platform parses and classifies the
// data; arch receives the typed source and operates on it directly.
pub enum BootSource<'a> {
    Fdt(&'a fdt::Fdt<'a>),
    Acpi,     // TODO: add ACPI table type when implementing
    Firmware, // TODO: add firmware-specific type when implementing
}
