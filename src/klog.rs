use crate::println;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

struct KernelLogger;

impl log::Log for KernelLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level().to_level().unwrap()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            match record.level() {
                Level::Debug | Level::Trace => println!(
                    "[{} from {}:{}] {}",
                    record.level(),
                    record.file().unwrap_or("unknown source"),
                    record.line().unwrap_or_default(),
                    record.args()
                ),
                _ => println!("[{}] {}", record.level(), record.args()),
            }
            // println!("{:#?}", record);
        }
    }

    fn flush(&self) {}
}

static LOGGER: KernelLogger = KernelLogger;

#[cfg(debug_assertions)]
pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Debug))
}

#[cfg(not(debug_assertions))]
pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Warn))
}
