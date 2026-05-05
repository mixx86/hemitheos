use hemitheos::platform;

#[test]
#[should_panic(expected = "DTB address is null")]
fn parse_dtb_rejects_null() {
    platform::parse_dtb(0);
}

#[test]
#[should_panic(expected = "Invalid DTB address")]
fn parse_dtb_rejects_bad_magic() {
    // A non-null aligned address whose first four bytes are not 0xD00DFEED.
    static ZERO: u32 = 0;
    platform::parse_dtb((&ZERO as *const u32) as usize);
}

#[test]
#[should_panic(expected = "platform not initialized")]
fn platform_before_init_panics() {
    let _ = platform::platform();
}
