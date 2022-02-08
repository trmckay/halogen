use lazy_static::lazy_static;
pub use log::*;

lazy_static! {
    pub static ref LOGGER: UartLogger = UartLogger;
}

pub struct UartLogger;

impl log::Log for UartLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            crate::println!("{}: {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

impl UartLogger {
    pub fn register(&'static self) {
        log::set_logger(self)
            .map(|_| log::set_max_level(LevelFilter::Info))
            .expect("Failed to initialize logger")
    }
}
