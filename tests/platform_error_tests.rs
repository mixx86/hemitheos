use hemitheos::platform;

#[test]
#[should_panic(expected = "no platform information source available")]
fn init_rejects_null() {
    platform::init(0);
}

#[test]
#[should_panic(expected = "no platform information source available")]
fn init_rejects_bad_magic() {
    // A non-null aligned address whose first four bytes are not 0xD00DFEED.
    static ZERO: u32 = 0;
    platform::init((&ZERO as *const u32) as usize);
}

#[test]
#[should_panic(expected = "platform not initialized")]
fn platform_before_init_panics() {
    let _ = platform::platform();
}
