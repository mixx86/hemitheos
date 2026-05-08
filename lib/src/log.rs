use core::cell::UnsafeCell;
use core::fmt::Write;
use core::sync::atomic::{AtomicU8, AtomicUsize, Ordering};

use crate::{console, platform};

#[derive(Clone, Copy)]
pub enum LogLevel {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
}

#[cfg(debug_assertions)]
pub const MAX_LOG_LEVEL: LogLevel = LogLevel::Debug;
#[cfg(not(debug_assertions))]
pub const MAX_LOG_LEVEL: LogLevel = LogLevel::Info;

const LOG_BUF_SIZE: usize = 8192;

pub struct LogBuffer {
    buf: UnsafeCell<[u8; LOG_BUF_SIZE]>,
    head: AtomicUsize,
    tail: AtomicUsize,
}

unsafe impl Sync for LogBuffer {}

impl LogBuffer {
    pub const fn new() -> Self {
        Self {
            buf: UnsafeCell::new([0; LOG_BUF_SIZE]),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    pub fn write(&self, s: &[u8]) {
        let mut head = self.head.load(Ordering::Acquire);
        let mut tail = self.tail.load(Ordering::Relaxed);
        let buf = self.buf.get();

        for &b in s {
            while (tail + 1) % LOG_BUF_SIZE == head {
                head = (head + 1) % LOG_BUF_SIZE;
                self.head.store(head, Ordering::Relaxed);
            }
            unsafe {
                (*buf)[tail] = b;
            }
            tail = (tail + 1) % LOG_BUF_SIZE;
        }
        self.tail.store(tail, Ordering::Release);
    }

    pub fn read(&self, out: &mut [u8]) -> usize {
        let mut head = self.head.load(Ordering::Acquire);
        let tail = self.tail.load(Ordering::Acquire);
        let buf = unsafe { &*self.buf.get() };
        let mut count = 0;

        while head != tail && count < out.len() {
            out[count] = buf[head];
            head = (head + 1) % LOG_BUF_SIZE;
            count += 1;
        }
        self.head.store(head, Ordering::Release);
        count
    }
}

const EMPTY: LogBuffer = LogBuffer::new();
static LOG_BUFFERS: [LogBuffer; platform::MAX_SMP_CPUS] = [EMPTY; platform::MAX_SMP_CPUS];

fn current_log_buffer() -> &'static LogBuffer {
    let cpu_id = platform::current_cpu_id();

    &LOG_BUFFERS[cpu_id]
}

static CONSOLE_LOG_LEVEL: AtomicU8 = AtomicU8::new(LogLevel::Debug as u8);

struct LogWriter(&'static LogBuffer, LogLevel);

pub fn set_level(level: LogLevel) {
    CONSOLE_LOG_LEVEL.store(level as u8, Ordering::Relaxed);
}

impl core::fmt::Write for LogWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if self.1 as u8 <= CONSOLE_LOG_LEVEL.load(Ordering::Relaxed) {
            console::puts(s);
        }

        self.0.write(s.as_bytes());
        Ok(())
    }
}

pub fn log_write(level: LogLevel, args: core::fmt::Arguments) {
    let mut writer = LogWriter(current_log_buffer(), level);
    let _ = write!(writer, "{}", args);
}

macro_rules! log_error {
      ($($arg:tt)*) => {{
            $crate::log::log_write($crate::log::LogLevel::Error, format_args!($($arg)*));
      }};
}

macro_rules! log_warn {
      ($($arg:tt)*) => {{
          if ($crate::log::LogLevel::Warn as u8) <= ($crate::log::MAX_LOG_LEVEL as u8) {
            $crate::log::log_write($crate::log::LogLevel::Warn, format_args!($($arg)*));
          }
      }};
}

macro_rules! log_info {
      ($($arg:tt)*) => {{
          if ($crate::log::LogLevel::Info as u8) <= ($crate::log::MAX_LOG_LEVEL as u8) {
              $crate::log::log_write($crate::log::LogLevel::Info, format_args!($($arg)*));
          }
      }};
}

macro_rules! log_debug {
      ($($arg:tt)*) => {{
          if ($crate::log::LogLevel::Debug as u8) <= ($crate::log::MAX_LOG_LEVEL as u8) {
            $crate::log::log_write($crate::log::LogLevel::Debug, format_args!($($arg)*));
          }
      }};
}

macro_rules! print {
    ($($arg:tt)*) => { log_info!($($arg)*) };
}

macro_rules! println {
      ($($arg:tt)*) => { log_info!("{}\n", format_args!($($arg)*)) };
}
