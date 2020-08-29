use crate::serial_println;
use log::{LevelFilter, Metadata, Record, SetLoggerError};
use crate::textbuffer::{Textbuffer};
use spin::Mutex;
use core::fmt;
use lazy_static::lazy_static;
use alloc::string::String;

lazy_static! {
    pub static ref LOG_BUFFER: Mutex<Textbuffer> = Mutex::new(Textbuffer::new());
}

struct KernelLogger;

impl log::Log for KernelLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level().to_level().unwrap()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            serial_println!(
                "[{:<5} from {:>25}:{:<3} at {:>5}] {}",
                record.level(),
                record.file().unwrap_or("unknown source"),
                record.line().unwrap_or_default(),
                crate::time::get(),
                record.args()
            );
            if let Some(true) = crate::allocator::HEAP_INITIALIZED.get() {
                let mut log_buffer = LOG_BUFFER.lock();
                let mut string = String::new();
                fmt::write(&mut string, format_args!(
                    "[{:<5} from {:>25}:{:<3} at {:>5}] {}",
                    record.level(),
                    record.file().unwrap_or("unknown source"),
                    record.line().unwrap_or_default(),
                    crate::time::get(),
                    record.args()
                )).expect("error converting fmt::Arguments to String");
                log_buffer.write_string(&string);
                log_buffer.new_line();
            }
        }
    }

    fn flush(&self) {
        LOG_BUFFER.lock().flush();
    }
}

static LOGGER: KernelLogger = KernelLogger;

#[cfg(debug_assertions)]
pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::max()))
}

#[cfg(not(debug_assertions))]
pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Warn))
}
