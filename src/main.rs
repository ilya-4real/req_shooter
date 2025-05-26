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

use url_parser::ParsedUrlAndHeader;

fn run_pool(url: &str, header: Option<String>, duration: usize, threads: u8, connections: usize) {
    let parsed_url = ParsedUrlAndHeader::parse_url(url);
    let mut ready_url_and_header: ParsedUrlAndHeader;
    match parsed_url {
        Err(e) => {
            println!("Url parsing error: {}", e);
            return;
        }
        Ok(url) => {
            ready_url_and_header = url;
        }
    };
    if header.is_some() {
        ready_url_and_header
            .add_header(header.unwrap())
            .expect("invalid header provided");
    }
    let job = MioHTTPJob {
        parsed_url: ready_url_and_header.clone(),
        job_duration_sec: duration,
        conn_quantity: connections,
    };
    let th_pool: ThreadPool = ThreadPool::new(threads);
    if let Err(_) = th_pool.start(Box::new(job)) {
        println!("Got error while running the threadpool. This error caused by previous errors")
    }
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
