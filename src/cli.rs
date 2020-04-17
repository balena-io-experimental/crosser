use clap::{Arg, ArgMatches};

pub struct CliArgs {
    pub config: String,
    pub token: String,
}

pub fn read_cli_args() -> CliArgs {
    let default_config = format!("{}.yml", env!("CARGO_PKG_NAME"));

    let matches = app_from_crate!()
        .arg(
            Arg::with_name("CONFIG")
                .short("c")
                .long("config")
                .value_name("config")
                .env("CROSSER_CONFIG")
                .help("Config file")
                .takes_value(true)
                .default_value(&default_config),
        )
        .arg(
            Arg::with_name("TOKEN")
                .short("t")
                .long("token")
                .value_name("token")
                .env("CROSSER_TOKEN")
                .help("Access token")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let config = get_existing_arg(&matches, "CONFIG");
    let token = get_existing_arg(&matches, "TOKEN");

    CliArgs { config, token }
}

fn get_existing_arg(matches: &ArgMatches, name: &str) -> String {
    if let Some(contents) = matches.value_of(name) {
        contents.into()
    } else {
        unreachable!()
    }
}
