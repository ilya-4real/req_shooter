mod cli_args;
mod jobs;
mod threadpool;
mod url_parser;
pub mod utils;
use std::time::Instant;

use clap::Parser;
use jobs::http_job::HTTPJob;
use threadpool::ThreadPool;

use url_parser::ParsedUrl;

fn run_pool() {
    let parsed_url = ParsedUrl::new("localhost:8000/").expect("can not parse url");
    let job1: HTTPJob = HTTPJob {
        parsed_url: parsed_url.clone(),
        job_duration_sec: 5,
        conn_quantity: 200,
    };
    let th_pool: ThreadPool = ThreadPool::new(4);
    th_pool.start(Box::new(job1));
}

fn main() {
    let start_time = Instant::now();
    run_pool();
    println!("time spent: {:?}", start_time.elapsed());

    let args = cli_args::CliArgs::parse();
    println!("{args:?}");
}
