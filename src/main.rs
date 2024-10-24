mod cli_args;
pub mod http_parser;
mod jobs;
mod statistics;
mod threadpool;
mod url_parser;
pub mod utils;

use cli_args::get_parsed_args;
use jobs::mio_job::MioHTTPJob;
use threadpool::ThreadPool;

use url_parser::ParsedUrl;

fn run_pool(url: &str, duration: usize, threads: u8, connections: usize) {
    let parsed_url = ParsedUrl::new(url).expect("can not parse url");
    let job2 = MioHTTPJob {
        parsed_url: parsed_url.clone(),
        job_duration_sec: duration,
        conn_quantity: connections,
    };
    let th_pool: ThreadPool = ThreadPool::new(threads);
    th_pool.start(Box::new(job2));
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
