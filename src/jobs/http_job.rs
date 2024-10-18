use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

use crate::statistics::stats::WorkerStats;
use crate::url_parser::ParsedUrl;
use std::collections::VecDeque;

use super::job::{CloneJob, Job};

enum HTTPSendResult {
    Error,
    ConnectionClosed,
    Successful,
}

enum HTTPReadResult {
    Successful,
    NonSuccessfulCode,
}

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
    fn execute(&self, stats_sender: Sender<WorkerStats>) {
        println!(
            "\tshooting at : host -> {}, port -> {}",
            self.parsed_url.host, self.parsed_url.port
        );

        let mut conn_q: VecDeque<TcpStream> = VecDeque::with_capacity(self.conn_quantity);

        fill_conn_q(&self.parsed_url, &mut conn_q, self.conn_quantity);
        let start_time = Instant::now();
        let time_of_end = Instant::now() + Duration::from_secs(self.job_duration_sec as u64);
        let mut latencies: Vec<f64> = Vec::new();
        let mut total_requests: i64 = 0;
        let mut total_errors: i64 = 0;
        let mut bad_requests: i64 = 0;
        let request = format!(
            "GET /{} HTTP/1.1\r\nHost: {}\r\n\r\n",
            self.parsed_url.resource, self.parsed_url.host
        );
        let mut worker_statistics = WorkerStats::new();
        loop {
            if Instant::now() >= time_of_end {
                break;
            } else {
                let mut connection = conn_q.pop_front().expect("no connection found in queue");
                let request_send_start_time = Instant::now();
                match send_http_request(&request.as_bytes(), &mut connection) {
                    HTTPSendResult::ConnectionClosed => {
                        fill_conn_q(&self.parsed_url, &mut conn_q, 1);
                    }
                    HTTPSendResult::Error => {
                        total_errors += 1;
                    }
                    HTTPSendResult::Successful => {
                        match read_response(&mut connection) {
                            HTTPReadResult::NonSuccessfulCode => bad_requests += 1,
                            HTTPReadResult::Successful => {}
                        }
                        conn_q.push_back(connection);
                        total_requests += 1;
                        latencies.push(request_send_start_time.elapsed().as_micros() as f64);
                    }
                }
            }
        }

        worker_statistics.set_request_count(total_requests as usize);
        worker_statistics.set_duration(start_time.elapsed().as_secs() as usize);
        worker_statistics.set_error_count(total_errors as usize);
        worker_statistics.set_bad_requests(bad_requests as usize);
        worker_statistics.calculate(latencies);
        let _ = stats_sender.send(worker_statistics).unwrap();
    }
}

fn fill_conn_q(parsed_url: &ParsedUrl, conn_q: &mut VecDeque<TcpStream>, capacity: usize) {
    let addr = format!("{}:{}", parsed_url.host, parsed_url.port);
    let filling_start_time = Instant::now();
    for _ in 0..capacity {
        conn_q.push_back(TcpStream::connect(&addr).unwrap());
    }
    if filling_start_time.elapsed().as_secs() > 1 {
        println!("WARNING! server accepts connections too slowly. try to reduce active connections")
    }
}

fn send_http_request(request: &[u8], connection: &mut TcpStream) -> HTTPSendResult {
    println!("sending");
    match connection.write_all(request) {
        Err(e) => {
            if e.kind() == ErrorKind::BrokenPipe {
                return HTTPSendResult::ConnectionClosed;
            } else {
                return HTTPSendResult::Error;
            }
        }
        Ok(_sent) => HTTPSendResult::Successful,
    }
}

fn read_response(conn: &mut TcpStream) -> HTTPReadResult {
    println!("reading");
    let mut response = String::new();
    let mut buffer = [0; 1024];
    let mut content_length: usize = 0;
    let mut headers_complete = false;

    loop {
        let bytes_read = conn.read(&mut buffer).unwrap();

        if bytes_read == 0 {
            break;
        }

        response.push_str(std::str::from_utf8(&buffer[..bytes_read]).unwrap());
        if !headers_complete {
            if let Some(header) = response.lines().find(|l| l.starts_with("Content-Length: ")) {
                content_length = header.split(": ").nth(1).unwrap().parse().unwrap();
                headers_complete = true;
            }
        }
        if headers_complete && response.len() >= content_length + headers_complete as usize {
            break;
        }
    }
    let response_status_code_symbol = response.chars().nth(9).unwrap();
    println!("{response}");
    // println!("{}", response_status_code_symbol);
    if response_status_code_symbol != '2' && response_status_code_symbol != '3' {
        HTTPReadResult::NonSuccessfulCode
    } else {
        HTTPReadResult::Successful
    }
}
