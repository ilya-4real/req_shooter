mod cli_args;
mod jobs;
mod threadpool;
mod url_parser;
pub mod utils;
use std::time::Instant;
mod statistics;
use clap::{command, Arg, ArgMatches};
use cli_args::get_parsed_args;
use jobs::http_job::HTTPJob;
use threadpool::ThreadPool;

use url_parser::ParsedUrl;

fn run_pool(url: &str, duration: usize, threads: u8, connections: usize) {
    let parsed_url = ParsedUrl::new(url).expect("can not parse url");
    let job1: HTTPJob = HTTPJob {
        parsed_url: parsed_url.clone(),
        job_duration_sec: duration,
        conn_quantity: connections,
    };
    let th_pool: ThreadPool = ThreadPool::new(threads);
    th_pool.start(Box::new(job1));
}

fn main() {
    let cli_args = get_parsed_args();
    run_pool(
        &cli_args.url,
        cli_args.duration,
        cli_args.threads,
        cli_args.connections,
    );
}
