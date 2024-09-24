use std::io::{Read, Write};
use std::net::TcpStream;

use crate::url_parser::ParsedUrl;

pub fn make_http_request(parsed_url: &ParsedUrl) {
    let addr = format!("{}:{}", parsed_url.host, parsed_url.port);
    let mut tcp_stream: TcpStream = TcpStream::connect(addr).unwrap();
    let request = "GET / HTTP/1.1\r\nHost: ilya4r.ru\r\n\r\n";
    println!("request: {request:?}");
    let _write_res = tcp_stream
        .write_all(request.as_bytes())
        .expect("unable to write  a message into tcp connection");

    let mut buffer = [0; 1024];
    let bytes_read = tcp_stream
        .read(&mut buffer)
        .expect("unable to read message from tcp stream");
    let response = String::from_utf8_lossy(&buffer[0..bytes_read]);
    println!("response: {}", response);
}
