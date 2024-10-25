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

use url_parser::ParsedUrlHeader;

fn run_pool(url: &str, header: Option<String>, duration: usize, threads: u8, connections: usize) {
    let mut parsed_url = ParsedUrlHeader::parse_url(url).expect("can not parse url");
    if header.is_some() {
        parsed_url
            .add_header(header.unwrap())
            .expect("invalid header provided");
    }
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
        cli_args.header,
        cli_args.duration,
        cli_args.threads,
        cli_args.connections,
    );
}
