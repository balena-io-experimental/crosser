use std::path::Path;

use anyhow::{Context, Result};

use log::{LevelFilter, Log, Metadata, Record};

static LOGGER: Logger = Logger;

struct Logger;

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let origin = get_origin(record);
        let trimmed = &origin[..std::cmp::min(origin.len(), 8)];
        let with_brackets = format!("[{}]", trimmed);
        println!("{:10} {}", with_brackets, record.args());
    }

    fn flush(&self) {}
}

fn get_origin(record: &Record) -> String {
    if let Some(module) = record.module_path() {
        if module.starts_with("crosser") {
            if let Some(file) = record.file() {
                let path = Path::new(file);
                let file_stem = path
                    .file_stem()
                    .map(|os_str| os_str.to_string_lossy())
                    .unwrap_or_default();
                return file_stem.to_string();
            }
        }
        return module.to_string();
    }
    return "...".to_string();
}

pub fn init() -> Result<()> {
    log::set_logger(&LOGGER).context("Logging initialization failed")?;
    log::set_max_level(LevelFilter::Info);
    Ok(())
}
