/* Fixmap slots */

// Main UART page/s, either for early UART data or
// for the data from the UART found in DTB.
// Both are using in this slot, for the future we want to move
// the place where we store data from the DTB UART, like for example
// in ioremap.
// TODO: ioremap for DTB UART
pub const FIX_CONSOLE: u8 = 0;
