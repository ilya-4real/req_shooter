use std::{
    collections::HashMap,
    io::{ErrorKind, Read, Write},
    net::SocketAddr,
    time::Instant,
};

use mio::{net::TcpStream, Events, Interest, Poll, Token};

use crate::{statistics::stats::WorkerStats, url_parser::ParsedUrl};

use super::job::{CloneJob, Job};

#[derive(Debug)]
enum ConnectionState {
    Connected,
    HeadersRead,
    Closed,
}

struct HTTPConnection {
    tcp_stream: TcpStream,
    state: ConnectionState,
    request_counter: u32,
    request_sent_time: Option<Instant>,
}

impl HTTPConnection {
    fn new(tcp_address: std::net::SocketAddr) -> HTTPConnection {
        let new_stream = TcpStream::connect(tcp_address)
            .expect("unable to establish tcp connection. check if the server is available");
        return HTTPConnection {
            tcp_stream: new_stream,
            state: ConnectionState::Connected,
            request_counter: 0,
            request_sent_time: None,
        };
    }

    fn read_available(&mut self) {
        let mut buffer = [0; 4096];
        match self.tcp_stream.read(&mut buffer) {
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    return;
                } else {
                    println!("{e}");
                }
            }
            Ok(_n) => {
                let response_string = std::str::from_utf8(&buffer).unwrap();
                let mut headers: Vec<&str> = Vec::new();
                for line in response_string.lines() {
                    headers.push(line);
                    if line == "\r\n" {
                        self.state = ConnectionState::HeadersRead;
                    }
                }
                self.request_counter += 1;
            }
        }
    }

    fn send_request(&mut self, request: &[u8]) {
        match self.tcp_stream.write_all(request) {
            Ok(_) => {
                self.request_sent_time = Some(Instant::now());
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::BrokenPipe {
                    self.state = ConnectionState::Closed
                } else {
                    println!("{e}");
                }
            }
        }
    }
}

fn register_new_socket(
    poll: &mut Poll,
    token: Token,
    socket_address: SocketAddr,
) -> HTTPConnection {
    let mut new_connection = HTTPConnection::new(socket_address);
    poll.registry()
        .register(
            &mut new_connection.tcp_stream,
            token,
            Interest::READABLE | Interest::WRITABLE,
        )
        .expect("unable to register socket");
    new_connection
}

fn fill_connection_map(
    size: usize,
    map: &mut HashMap<Token, HTTPConnection>,
    poll: &mut Poll,
    socket_address: SocketAddr,
) {
    for i in 0..size {
        let new_token = Token(i);
        map.insert(
            new_token,
            register_new_socket(poll, new_token, socket_address),
        );
    }
}
fn reregister_socket_in_map(
    token: Token,
    map: &mut HashMap<Token, HTTPConnection>,
    poll: &mut Poll,
    socket_address: SocketAddr,
) {
    map.insert(token, register_new_socket(poll, token, socket_address));
}

#[derive(Clone)]
pub struct MioHTTPJob {
    pub parsed_url: ParsedUrl,
    pub job_duration_sec: usize,
    pub conn_quantity: usize,
}

impl CloneJob for MioHTTPJob {
    fn clone_job<'a>(&self) -> Box<dyn Job + Send + Sync> {
        Box::new(self.clone())
    }
}

impl Job for MioHTTPJob {
    fn execute(
        &self,
        stats_sender: std::sync::mpsc::Sender<crate::statistics::stats::WorkerStats>,
    ) {
        let mut poll = Poll::new().expect("unable to create poll");
        let mut events = Events::with_capacity(self.conn_quantity);
        let mut connection_map: HashMap<Token, HTTPConnection> = HashMap::new();
        let request = format!(
            "GET /{} HTTP/1.1\r\nHost: {}\r\n\r\n",
            self.parsed_url.resource, self.parsed_url.host
        );
        let socket_address: SocketAddr =
            format!("{}:{}", self.parsed_url.host, self.parsed_url.port)
                .parse()
                .expect("unable to parse socket adress");
        fill_connection_map(
            self.conn_quantity,
            &mut connection_map,
            &mut poll,
            socket_address,
        );
        let mut request_count = 0;
        let mut bad_requests = 0;
        let mut errors = 0;
        let mut latencies = vec![0f64; 0];
        let start_time = Instant::now();
        loop {
            if start_time.elapsed().as_secs() >= self.job_duration_sec as u64 {
                break;
            }
            poll.poll(&mut events, None)
                .expect("can not execute poll operation");
            for event in &events {
                let token = event.token();
                let connection = connection_map.get_mut(&token).unwrap();
                if event.is_readable() {
                    latencies
                        .push(connection.request_sent_time.unwrap().elapsed().as_micros() as f64);
                    connection.read_available();
                }
                if event.is_writable() {
                    connection.send_request(request.as_bytes());
                }
                if event.is_read_closed() || event.is_write_closed() {
                    request_count += connection.request_counter;
                    reregister_socket_in_map(token, &mut connection_map, &mut poll, socket_address);
                }
            }
        }
        for connection in connection_map.values() {
            request_count += connection.request_counter
        }
        let mut worker_statistics = WorkerStats::new();
        worker_statistics.set_duration(self.job_duration_sec);
        worker_statistics.set_request_count(request_count as usize);
        worker_statistics.set_bad_requests(bad_requests as usize);
        worker_statistics.set_error_count(errors as usize);
        worker_statistics.calculate(latencies);
        stats_sender.send(worker_statistics).unwrap();
    }
}
