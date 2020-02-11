use clap::{App, Arg, ArgMatches};

pub struct Args {
    pub token: String,
}

pub fn get_cli_args() -> Args {
    let matches = App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("TOKEN")
                .help("Balena API token")
                .required(true)
                .index(1),
        )
        .get_matches();

    let token = get_token(&matches);

    Args { token: token }
}

fn get_token(matches: &ArgMatches) -> String {
    if let Some(contents) = matches.value_of("TOKEN") {
        contents.into()
    } else {
        unreachable!()
    }
}
