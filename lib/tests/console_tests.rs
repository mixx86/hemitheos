use hemitheos::console;

#[test]
fn puts_before_init_is_noop() {
    console::puts("ignored");
}

#[test]
fn init_with_none_is_noop() {
    console::init(None);
}

#[test]
#[should_panic(expected = "platform not initialized")]
fn console_before_init_panics() {
    let _ = console::console();
}
