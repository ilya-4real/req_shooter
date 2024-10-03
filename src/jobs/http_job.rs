use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::Sender;
use std::thread::sleep;
use std::time::{Duration, Instant};

use crate::statistics::stats::Statistics;
use crate::url_parser::ParsedUrl;
use std::collections::VecDeque;

use super::job::{CloneJob, Job};

#[derive(Clone)]
pub struct HTTPJob {
    pub parsed_url: ParsedUrl,
    pub job_duration_sec: usize,
    pub conn_quantity: usize,
}
impl CloneJob for HTTPJob {
    fn clone_job<'a>(&self) -> Box<dyn Job + Send + Sync> {
        Box::new(self.clone())
    }
}

impl Job for HTTPJob {
    fn execute(&self, stats_sender: Sender<Statistics>) {
        println!(
            "shooting on : host -> {}, port -> {}",
            self.parsed_url.host, self.parsed_url.port
        );
        let start_time = Instant::now();
        let mut conn_q: VecDeque<TcpStream> = VecDeque::with_capacity(self.conn_quantity);

        fill_conn_q(&self.parsed_url, &mut conn_q, self.conn_quantity);

        let time_of_end = Instant::now() + Duration::from_secs(self.job_duration_sec as u64);
        let mut total_requests: i64 = 0;
        let mut total_errors: i64 = 0;
        let mut bad_requests: i64 = 0;
        let request = format!(
            "GET /{} HTTP/1.1\r\nHost: {}\r\n\r\n",
            self.parsed_url.resource, self.parsed_url.host
        );
        let mut local_statistics = Statistics::new();
        loop {
            if Instant::now() >= time_of_end {
                break;
            } else {
                match make_http_request(request.as_bytes(), &mut conn_q) {
                    HTTPResult::Successful => total_requests += 1,
                    HTTPResult::Error => total_errors += 1,
                    HTTPResult::NonSuccessfulCode => bad_requests += 1,
                    HTTPResult::ConnectionClosed => {
                        fill_conn_q(&self.parsed_url, &mut conn_q, 1);
                    }
                }
            }
        }
        local_statistics.set_request_count(total_requests as usize);
        local_statistics.set_duration(start_time.elapsed().as_secs() as usize);
        local_statistics.set_error_count(total_errors as usize);
        local_statistics.set_bad_requests(bad_requests as usize);
        let _ = stats_sender.send(local_statistics).unwrap();
        println!("requests made {}", total_requests);
        println!("job completed in: {:?}", start_time.elapsed());
    }
}

enum HTTPResult {
    Successful,
    Error,
    NonSuccessfulCode,
    ConnectionClosed,
}

fn fill_conn_q(parsed_url: &ParsedUrl, conn_q: &mut VecDeque<TcpStream>, capacity: usize) {
    let addr = format!("{}:{}", parsed_url.host, parsed_url.port);
    for i in 0..capacity {
        println!("added into q: {}", i);
        conn_q.push_back(TcpStream::connect(&addr).unwrap());
    }
}

fn is_response_2xx_or_3xx(response: &str) -> bool {
    match response.chars().nth(9) {
        Some(ch) => return ch == '2' || ch == '3',
        None => {
            println!("{response}");
            return false;
        }
    }
}

fn make_http_request(request: &[u8], conn_q: &mut VecDeque<TcpStream>) -> HTTPResult {
    let mut conn = conn_q.pop_front().unwrap();
    // println!("stop point before writing");
    let write_res = conn.write_all(request);
    match write_res {
        Ok(_res) => {
            let mut buffer = [0; 1500];
            // println!("stop point before reading");
            let bytes_read = conn
                .read(&mut buffer)
                .expect("unable to read message from tcp stream");
            conn_q.push_back(conn);
            let response = String::from_utf8_lossy(&buffer[0..bytes_read]).to_string();
            // println!("{response}");
            if is_response_2xx_or_3xx(&response) {
                return HTTPResult::Successful;
            }
            return HTTPResult::NonSuccessfulCode;
        }
        Err(err) => {
            if err.kind() == std::io::ErrorKind::BrokenPipe {
                return HTTPResult::ConnectionClosed;
            } else {
                return HTTPResult::Error;
            }
        }
    };
}
