use std::io::{stdout, Write};
use std::path::Path;

use anyhow::{Context, Result};

use log::{LevelFilter, Log, Metadata, Record};

use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
};

static LOGGER: Logger = Logger;

struct Logger;

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let mut origin = get_origin(record);
        make_ascii_titlecase(&mut origin);
        let formatted_origin = format_origin(&origin);

        let _ = execute!(
            stdout(),
            SetForegroundColor(Color::Cyan),
            Print(formatted_origin),
            ResetColor,
            Print(' '),
            Print(record.args()),
            Print('\n')
        );
    }

    fn flush(&self) {}
}

fn format_origin(origin: &str) -> String {
    let trimmed = &origin[..std::cmp::min(origin.len(), 8)];
    let with_brackets = format!("[{}]", trimmed);
    let with_spaces = format!("{:10}", with_brackets);
    with_spaces
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

fn make_ascii_titlecase(s: &mut str) {
    if let Some(r) = s.get_mut(0..1) {
        r.make_ascii_uppercase();
    }
}

pub fn init() -> Result<()> {
    log::set_logger(&LOGGER).context("Logging initialization failed")?;
    log::set_max_level(LevelFilter::Info);
    Ok(())
}
