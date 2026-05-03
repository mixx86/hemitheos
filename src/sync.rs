use core::cell::UnsafeCell;
use core::prelude::v1::*;
use core::sync::atomic::{AtomicU8, Ordering};

const ONCE_UNINIT: u8 = 0;
const ONCE_WRITING: u8 = 1;
const ONCE_READY: u8 = 2;

struct Once<T> {
    state: AtomicU8,
    data: UnsafeCell<Option<T>>,
}

// SAFETY: T: Send guarantees it's safe to share across threads once written.
unsafe impl<T: Send> Sync for Once<T> {}

impl<T> Once<T> {
    pub const fn new() -> Self {
        Self {
            state: AtomicU8::new(ONCE_UNINIT),
            data: UnsafeCell::new(None),
        }
    }

    pub fn set(&self, value: T) {
        if self
            .state
            .compare_exchange(
                ONCE_UNINIT,
                ONCE_WRITING,
                Ordering::Acquire,
                Ordering::Relaxed,
            )
            .is_err()
        {
            panic!("Once::set called more than once");
        }
        // SAFETY: CAS gave us exclusive access; data is written before READY state
        unsafe { *self.data.get() = Some(value) };
        self.state.store(ONCE_READY, Ordering::Release);
    }

    pub fn get(&self) -> Option<&T> {
        if self.state.load(Ordering::Acquire) == ONCE_READY {
            // SAFETY: READY is only set after data is fully written (Release/Acquire pair)
            unsafe { (*self.data.get()).as_ref() }
        } else {
            None
        }
    }
}
