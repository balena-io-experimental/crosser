use anyhow::{Context, Result};

use log::{LevelFilter, Log, Metadata, Record};

static LOGGER: Logger = Logger;

struct Logger;

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        println!("{}", record.args());
    }

    fn flush(&self) {}
}

pub fn init() -> Result<()> {
    log::set_logger(&LOGGER).context("Logging initialization failed")?;
    log::set_max_level(LevelFilter::Trace);
    Ok(())
}
