use clap::Parser;

#[derive(Parser, Debug)]
pub struct CliArgs {
    pub url: String,
    pub threads: u8,
    pub duration: u8,
}
