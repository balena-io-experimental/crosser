use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use ron::de::from_reader;
use ron::ser::{to_string_pretty, PrettyConfig};

use serde::{Deserialize, Serialize};

use crate::cli::CliArgs;
use crate::device::DeviceRegistration;

#[derive(Debug, Deserialize, Serialize)]
pub struct State(HashMap<String, DeviceRegistration>);

impl State {
    pub fn new() -> Self {
        State(HashMap::new())
    }
}

pub fn load_state(cli_args: &CliArgs) -> Result<State> {
    let filename = get_state_filename(cli_args);

    let state = if let Ok(file) = File::open(&filename) {
        from_reader(file).context(format!("Deserializing state file '{:?}' failed", filename))?
    } else {
        State::new()
    };

    Ok(state)
}

pub fn add_device_registration(
    cli_args: &CliArgs,
    slug: &str,
    registration: &DeviceRegistration,
    state: &mut State,
) -> Result<()> {
    state.0.insert(slug.to_string(), registration.clone());

    save_state(cli_args, state)
}

pub fn get_device_registration(state: &State, slug: &str) -> Option<DeviceRegistration> {
    state.0.get(slug).cloned()
}

fn save_state(cli_args: &CliArgs, state: &State) -> Result<()> {
    let filename = get_state_filename(cli_args);

    let pretty = PrettyConfig::default();

    let serialized = to_string_pretty(state, pretty).context("State serialization failed")?;

    let mut file =
        File::create(&filename).context(format!("Creating state file '{:?}' failed", filename))?;

    file.write_all(serialized.as_bytes())
        .context("Unable to write state data")?;

    Ok(())
}

fn get_state_filename(args: &CliArgs) -> PathBuf {
    Path::new(&args.config).with_extension("state.ron")
}
