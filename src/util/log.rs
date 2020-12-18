use alloc::string::String;
pub use winapi::km::wdm::DbgPrint;
use winapi::km::wdm::DbgPrintEx;
use log::{Record, Metadata, SetLoggerError, Level, LevelFilter};

/// Prints a string using DbgPrintEx. Automatically adds a null terminator
fn __kernel_print(mut text: String) {
    text.push('\0');
    unsafe { DbgPrintEx(0, 0, text.as_ptr()) };
}

pub struct KernelLogger;

static LOGGER: KernelLogger = KernelLogger;

impl KernelLogger {
    pub fn init(level: LevelFilter) -> Result<(), SetLoggerError> {
        log::set_logger(&LOGGER)
            .map(|()| log::set_max_level(level))
    }
}

impl log::Log for KernelLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let prefix = match record.level() {
            Level::Error => "[ERROR]",
            Level::Warn => "[!]",
            Level::Info => "[+]",
            Level::Debug => "[*]",
            Level::Trace => "[?]",
        };

        __kernel_print(alloc::format!("{} {}", prefix, record.args()));
    }

    fn flush(&self) {}
}
