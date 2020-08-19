use crate::{println, serial_println};
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

struct KernelLogger;

impl log::Log for KernelLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level().to_level().unwrap()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            match record.level() {
                Level::Trace => {
                    serial_println!(
                        "[{} from {:>25}:{:<3} at {:>5}] {}",
                        record.level(),
                        record.file().unwrap_or("unknown source"),
                        record.line().unwrap_or_default(),
                        crate::time::get(),
                        record.args()
                    );
                }
                Level::Debug => {
                    serial_println!(
                        "[{} from {:>25}:{:<3} at {:>5}] {}",
                        record.level(),
                        record.file().unwrap_or("unknown source"),
                        record.line().unwrap_or_default(),
                        crate::time::get(),
                        record.args()
                    );
                    println!(
                        "[{} from {:>25}:{:<3} at {:>5}] {}",
                        record.level(),
                        record.file().unwrap_or("unknown source"),
                        record.line().unwrap_or_default(),
                        crate::time::get(),
                        record.args()
                    );
                },
                _ => {
                    serial_println!("[{}] {}", record.level(), record.args());
                    println!("[{}] {}", record.level(), record.args());
                },
            }
        }
    }

    fn flush(&self) {}
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
