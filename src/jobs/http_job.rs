use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::Sender;
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

        let time_off_end = std::time::Instant::now()
            .checked_add(Duration::from_secs(self.job_duration_sec as u64))
            .unwrap();
        let mut _total_requests: i64 = 0;
        let mut local_statistics = Statistics::new();
        while Instant::now() <= time_off_end {
            let request_result = make_http_request(&self.parsed_url.resource, &mut conn_q);
            match request_result {
                true => _total_requests += 1,
                false => fill_conn_q(&self.parsed_url, &mut conn_q, 1),
            }
        }
        local_statistics.set_request_count(_total_requests as usize);
        local_statistics.set_duration(start_time.elapsed().as_secs() as usize);
        let _ = stats_sender.send(local_statistics).unwrap();
        println!("requests made {}", _total_requests);
        println!("job completed in: {}s", start_time.elapsed().as_secs());
    }
}

fn fill_conn_q(parsed_url: &ParsedUrl, conn_q: &mut VecDeque<TcpStream>, capacity: usize) {
    let addr = format!("{}:{}", parsed_url.host, parsed_url.port);
    for _ in 0..capacity {
        conn_q.push_back(TcpStream::connect(&addr).unwrap());
    }
}

pub fn make_http_request(resource: &str, conn_q: &mut VecDeque<TcpStream>) -> bool {
    let mut conn = conn_q.pop_front().unwrap();
    let request = format!("GET /{} HTTP/1.1\r\nHost: ilya4r.ru\r\n\r\n", resource);
    let write_res = conn.write_all(request.as_bytes());
    match write_res {
        Ok(_res) => {
            let mut buffer = [0; 1024];
            let bytes_read = conn
                .read(&mut buffer)
                .expect("unable to read message from tcp stream");
            conn_q.push_back(conn);
            let _response = String::from_utf8_lossy(&buffer[0..bytes_read]);
            return true;
        }
        Err(_err) => return false,
    };
}
