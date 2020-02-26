use clap::{App, Arg};

pub struct CliArgs {
    pub config: String,
}

pub fn read_cli_args() -> CliArgs {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("CONFIG")
                .help("Config file")
                .required(false)
                .index(1),
        )
        .get_matches();

    let default_config = format!("{}.ron", env!("CARGO_PKG_NAME"));
    let config = matches
        .value_of("CONFIG")
        .unwrap_or(&default_config)
        .to_string();

    CliArgs { config }
}
