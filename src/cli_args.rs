use clap::{command, value_parser, Arg, ArgMatches};

pub struct CliArgs {
    pub threads: u8,
    pub header: Option<String>,
    pub connections: usize,
    pub duration: usize,
    pub url: String,
}

impl CliArgs {
    pub fn new(
        threads: u8,
        header: Option<String>,
        connections: usize,
        duration: usize,
        url: String,
    ) -> Self {
        return CliArgs {
            threads,
            header,
            connections,
            duration,
            url,
        };
    }
}

pub fn parse_cli_arguments() -> ArgMatches {
    let parsed_args = command!()
        .arg(
            Arg::new("threads")
                .short('t')
                .help("how many threads to run")
                .default_value("1")
                .value_parser(value_parser!(u8)),
        )
        .arg(
            Arg::new("header")
                .short('H')
                .help("header to send with request")
                .value_parser(value_parser!(String)),
        )
        .arg(
            Arg::new("conns")
                .short('c')
                .help("how many active connections to use in each thread")
                .default_value("100")
                .value_parser(value_parser!(usize)),
        )
        .arg(
            Arg::new("duration")
                .short('d')
                .help("how long to test in seconds")
                .required(true)
                .value_parser(value_parser!(usize)),
        )
        .arg(Arg::new("url").required(true))
        .get_matches();
    return parsed_args;
}

pub fn get_parsed_args() -> CliArgs {
    let argmatches = parse_cli_arguments();
    let threads = argmatches.get_one::<u8>("threads").unwrap();
    let connections = argmatches.get_one::<usize>("conns").unwrap();
    let duration = argmatches.get_one::<usize>("duration").unwrap();
    let set_header: Option<String> = match argmatches.get_one::<String>("header") {
        Some(header) => Some(header.clone()),
        None => None,
    };
    let url = argmatches
        .get_one::<String>("url")
        .expect("unable to parse url");

    return CliArgs::new(
        threads.clone(),
        set_header,
        connections.clone(),
        duration.clone(),
        url.clone(),
    );
}
