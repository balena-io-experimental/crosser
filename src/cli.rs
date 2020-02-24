use clap::{App, Arg};

pub struct CliArgs {
    pub config: Option<String>,
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

    let config = matches.value_of("CONFIG").map(|c| c.to_string());

    CliArgs { config }
}
