use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

const ONCE_UNINIT: u8 = 0;
const ONCE_WRITING: u8 = 1;
const ONCE_READY: u8 = 2;

pub struct Once<T> {
    state: AtomicU8,
    data: UnsafeCell<Option<T>>,
}

// SAFETY: T: Send guarantees it's safe to share across threads once written.
unsafe impl<T: Send> Sync for Once<T> {}

impl<T> Default for Once<T> {
    fn default() -> Self {
        Self::new()
    }
}

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

pub struct SpinLock {
    locked: AtomicBool,
}

impl Default for SpinLock {
    fn default() -> Self {
        Self::new()
    }
}

impl SpinLock {
    pub const fn new() -> Self {
        Self {
            locked: AtomicBool::new(false),
        }
    }
    pub fn lock(&self) {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            while self.locked.load(Ordering::Relaxed) {
                core::hint::spin_loop();
            }
        }
    }

    pub fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}
