mod cli_args;
pub mod http_request;
mod threadpool;
mod url_parser;
pub mod utils;
use clap::Parser;
use http_request::make_http_request;
use std::time::{Duration, Instant};
use threadpool::{HTTPJob, Job, ThreadPool};
use url_parser::ParsedUrl;

fn main() {
    let _th_pool: ThreadPool = ThreadPool::new(12);
    let parsed_url = ParsedUrl::new("localhost:8000/some_res").expect("can not parse url");
    make_http_request(&parsed_url);
    let cur_time = Instant::now();
    cur_time.checked_add(Duration::from_secs(5));

    let job: HTTPJob = HTTPJob {
        parsed_url,
        job_duration_sec: 5,
    };

    job.execute();

    let args = cli_args::CliArgs::parse();
    println!("{args:?}");
}
