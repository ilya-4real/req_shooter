use std::{
    io::{ErrorKind, Read, Write},
    net::{SocketAddr, ToSocketAddrs},
    time::Instant,
};

use mio::{net::TcpStream, Events, Interest, Poll, Token};
use slab::Slab;

use super::job::{CloneJob, Job};
use crate::http_parser::http_parser::{HTTParser, ParserState};
use crate::{statistics::stats::WorkerStats, url_parser::ParsedUrlHeader};

enum HTTPReadREsult {
    Complete(usize, char),
    Partial,
    Blocked,
    Error,
}

struct HTTPConnection {
    tcp_stream: TcpStream,
    parser: HTTParser,
    request_sent_time: Option<Instant>,
}

impl HTTPConnection {
    fn new(tcp_address: std::net::SocketAddr) -> HTTPConnection {
        let new_stream = TcpStream::connect(tcp_address)
            .expect("unable to establish tcp connection. check if the server is available");
        return HTTPConnection {
            tcp_stream: new_stream,
            parser: HTTParser::new(),
            request_sent_time: None,
        };
    }

    fn read_available(&mut self) -> HTTPReadREsult {
        let mut buffer = [0; 4096];
        match self.tcp_stream.read(&mut buffer) {
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock {
                    HTTPReadREsult::Blocked
                } else {
                    HTTPReadREsult::Error
                }
            }
            Ok(n) => {
                self.parser.parse(&buffer[..n]);
                match self.parser.state {
                    ParserState::Body => HTTPReadREsult::Partial,
                    ParserState::Started => {
                        HTTPReadREsult::Complete(n, self.parser.status_code_first_char)
                    }
                    _ => HTTPReadREsult::Error,
                }
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
                } else {
                    println!("{e}");
                }
            }
        }
    }
}

fn create_connection(socket_addr: SocketAddr) -> HTTPConnection {
    let connection = HTTPConnection::new(socket_addr);
    connection
}

fn fill_connection_slab(
    size: usize,
    socket_addr: SocketAddr,
    pool: &mut Slab<HTTPConnection>,
    poll: &mut Poll,
) {
    for _ in 0..size {
        let new_connection = create_connection(socket_addr);
        let token = pool.insert(new_connection);
        poll.registry()
            .register(
                &mut pool[token].tcp_stream,
                Token(token),
                Interest::WRITABLE | Interest::READABLE,
            )
            .expect("cannot not register socket");
    }
}
fn reregister_socket_in_slab(
    socket_addr: SocketAddr,
    token: Token,
    pool: &mut Slab<HTTPConnection>,
    poll: &mut Poll,
) {
    pool[token.0] = create_connection(socket_addr);
    poll.registry()
        .register(
            &mut pool[token.0].tcp_stream,
            token,
            Interest::WRITABLE | Interest::READABLE,
        )
        .expect("cannot register socket");
}

#[derive(Clone)]
pub struct MioHTTPJob {
    pub parsed_url: ParsedUrlHeader,
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
        &mut self,
        stats_sender: std::sync::mpsc::Sender<crate::statistics::stats::WorkerStats>,
    ) {
        let mut poll = Poll::new().expect("unable to create poll");
        let mut events = Events::with_capacity(self.conn_quantity);
        let mut connections_slab: Slab<HTTPConnection> = Slab::new();
        let request = self.parsed_url.compile_request();
        let socket_address = format!("{}:{}", self.parsed_url.host, self.parsed_url.port)
            .to_socket_addrs()
            .expect("can not resolve hostname")
            .next()
            .expect("there is no host with this name");
        println!("{}", socket_address);
        fill_connection_slab(
            self.conn_quantity,
            socket_address,
            &mut connections_slab,
            &mut poll,
        );
        let mut request_count: u32 = 0;
        let mut received_data = 0;
        let mut bad_requests: u32 = 0;
        let mut errors: u32 = 0;
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
                let connection = connections_slab.get_mut(token.0).unwrap();
                if event.is_readable() {
                    latencies
                        .push(connection.request_sent_time.unwrap().elapsed().as_micros() as f64);
                    match connection.read_available() {
                        HTTPReadREsult::Complete(response_size, status_first_char) => {
                            received_data += response_size;
                            if status_first_char != '2' && status_first_char != '3' {
                                bad_requests += 1
                            }
                        }
                        HTTPReadREsult::Error => {
                            errors += 1;
                        }
                        _ => {}
                    }
                }
                if event.is_writable() {
                    connection.send_request(request.as_bytes());
                }
                if event.is_read_closed() || event.is_write_closed() {
                    request_count += connection.parser.responses_parsed as u32;
                    reregister_socket_in_slab(
                        socket_address,
                        token,
                        &mut connections_slab,
                        &mut poll,
                    );
                }
            }
        }
        for (_, connection) in connections_slab {
            request_count += connection.parser.responses_parsed as u32
        }
        let mut worker_statistics = WorkerStats::new(
            self.job_duration_sec,
            request_count,
            errors,
            bad_requests,
            received_data,
        );
        worker_statistics.calculate_latencies(latencies);
        stats_sender.send(worker_statistics).unwrap();
    }
}
