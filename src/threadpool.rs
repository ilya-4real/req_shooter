use std::{
    thread::{spawn, JoinHandle, Thread},
    time::{self, Duration, Instant},
};

use crate::{http_request::make_http_request, url_parser::ParsedUrl};

#[derive(Debug)]
pub struct Worker {
    id: usize,
    thread: JoinHandle<()>,
}

impl Worker {
    pub fn new(id: usize) -> Worker {
        let thread = spawn(|| {});
        Worker { id, thread }
    }
}

pub struct HTTPJob {
    pub parsed_url: ParsedUrl,
    pub job_duration_sec: usize,
}

pub trait Job {
    fn execute(&self);
}

impl Job for HTTPJob {
    fn execute(&self) {
        let time_off_end = std::time::Instant::now()
            .checked_add(Duration::from_secs(self.job_duration_sec as u64))
            .unwrap();
        let mut _total_requests: i64 = 0;
        while Instant::now() <= time_off_end {
            make_http_request(&self.parsed_url);
            _total_requests += 1;
        }
        println!("requests made {}", _total_requests);
    }
}

#[derive(Debug)]
pub struct ThreadPool {
    workers_pool: Vec<Worker>,
}

impl ThreadPool {
    pub fn new(num_threads: usize) -> ThreadPool {
        let mut workers = Vec::with_capacity(num_threads);
        for id in 0..num_threads {
            workers.push(Worker::new(id));
        }
        ThreadPool {
            workers_pool: workers,
        }
    }
}
