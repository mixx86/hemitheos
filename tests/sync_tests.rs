use hemitheos::sync::Once;

#[test]
fn get_before_set_is_none() {
    let once: Once<u32> = Once::new();
    assert!(once.get().is_none());
}

#[test]
fn set_then_get_returns_value() {
    let once: Once<u32> = Once::new();
    once.set(42);
    assert_eq!(once.get(), Some(&42));
}

#[test]
fn get_is_stable() {
    let once: Once<u32> = Once::new();
    once.set(99);
    assert_eq!(once.get(), once.get());
}

#[test]
#[should_panic]
fn double_set_panics() {
    let once: Once<u32> = Once::new();
    once.set(1);
    once.set(2);
}
