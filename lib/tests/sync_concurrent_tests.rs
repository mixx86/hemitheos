use hemitheos::sync::Once;
use std::sync::{
    Arc, Barrier,
    atomic::{AtomicUsize, Ordering},
};
use std::thread;

const THREADS: usize = 8;

#[test]
fn once_concurrent_reads_after_set_are_consistent() {
    let once: Arc<Once<u64>> = Arc::new(Once::new());
    once.set(0xCAFE_BABE);

    let barrier = Arc::new(Barrier::new(THREADS));

    let handles: Vec<_> = (0..THREADS)
        .map(|_| {
            let once = Arc::clone(&once);
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                once.get().copied()
            })
        })
        .collect();

    for h in handles {
        assert_eq!(h.join().unwrap(), Some(0xCAFE_BABE_u64));
    }
}

#[test]
fn once_concurrent_set_exactly_one_wins() {
    use std::panic::AssertUnwindSafe;

    let once: Arc<Once<usize>> = Arc::new(Once::new());
    let wins = Arc::new(AtomicUsize::new(0));
    let barrier = Arc::new(Barrier::new(THREADS));

    let handles: Vec<_> = (0..THREADS)
        .map(|i| {
            let once = Arc::clone(&once);
            let wins = Arc::clone(&wins);
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                barrier.wait();
                if std::panic::catch_unwind(AssertUnwindSafe(|| once.set(i))).is_ok() {
                    wins.fetch_add(1, Ordering::Relaxed);
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    assert_eq!(wins.load(Ordering::Relaxed), 1);
    assert!(once.get().is_some());
}
