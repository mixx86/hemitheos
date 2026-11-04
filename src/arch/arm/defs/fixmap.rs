/* Fixmap slots */

// Main UART page/s, either for early UART data or
// for the data from the UART found in DTB.
// Both are using in this slot, for the future we want to move
// the place where we store data from the DTB UART, like for example
// in ioremap.
// TODO: ioremap for DTB UART
pub const FIX_CONSOLE: u8 = 0;

// Slots used by the PMAP to temporarily map page tables during early boot
// (see map_table() in mmu/pt.rs).
pub const FIX_PMAP_BEGIN: usize = 1;
pub const NUM_FIX_PMAP: usize = 4;
pub const FIX_PMAP_END: usize = FIX_PMAP_BEGIN + NUM_FIX_PMAP - 1;
