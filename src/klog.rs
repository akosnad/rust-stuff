use crate::println;
use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

struct KernelLogger;

impl log::Log for KernelLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level().to_level().unwrap()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!(
                "[{}] [{}]: {}",
                record.target(),
                record.level(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

static LOGGER: KernelLogger = KernelLogger;

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Warn))
}
