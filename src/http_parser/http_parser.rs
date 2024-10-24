use std::{collections::HashMap, io::Write};

#[derive(PartialEq, Debug)]
pub enum ParserState {
    Started,
    Version,
    StatusFirstChar,
    Status,
    HeaderStart,
    HeaderValue,
    HeaderEnd,
    Body,
    End,
}

pub struct HTTParser {
    pub state: ParserState,
    version: Vec<u8>,
    pub status_code_first_char: char,
    pub responses_parsed: usize,
    pub headers: HashMap<String, String>,
}

impl HTTParser {
    pub fn new() -> HTTParser {
        return HTTParser {
            state: ParserState::Started,
            version: vec![],
            status_code_first_char: '0',
            responses_parsed: 0,
            headers: HashMap::new(),
        };
    }

    pub fn parse(&mut self, data: &[u8]) {
        let mut current_header = String::new();
        let mut current_header_value = String::new();
        let mut body_size = 0;
        for byte in data {
            match self.state {
                ParserState::Started => {
                    if *byte == 47 {
                        // "/"
                        self.state = ParserState::Version
                    }
                }
                ParserState::Version => {
                    if *byte == 32 {
                        // space
                        self.state = ParserState::StatusFirstChar
                    } else {
                        self.version.push(*byte);
                    }
                }
                ParserState::StatusFirstChar => {
                    self.status_code_first_char = *byte as char;
                    self.state = ParserState::Status;
                }
                ParserState::Status => {
                    if *byte == 10 {
                        // '\n'
                        self.state = ParserState::HeaderStart
                    } else {
                        continue;
                    }
                }
                ParserState::HeaderStart => {
                    if *byte == 32 || *byte == 13 {
                        // space or \r
                        continue;
                    } else if *byte == 10 {
                        body_size = self
                            .headers
                            .get(&"content-length".to_string())
                            .unwrap_or(&"1".to_string())
                            .parse::<i32>()
                            .unwrap()
                            - 1;
                        match body_size {
                            0 => self.state = ParserState::Started,
                            _ => self.state = ParserState::Body,
                        }
                    } else if *byte == 58 {
                        // :
                        self.state = ParserState::HeaderValue;
                    } else {
                        current_header.push(*byte as char);
                    }
                }
                ParserState::HeaderValue => {
                    if *byte == 32 || *byte == 13 {
                        // space or \r
                        continue;
                    } else if *byte == 10 {
                        // \n
                        self.headers.insert(
                            current_header.to_lowercase().clone(),
                            current_header_value.clone(),
                        );
                        current_header.clear();
                        current_header_value.clear();
                        self.state = ParserState::HeaderStart;
                    } else {
                        current_header_value.push(*byte as char);
                    }
                }
                ParserState::Body => {
                    if body_size == 0 {
                        self.responses_parsed += 1;
                        self.state = ParserState::Started;
                    }
                    body_size -= 1;
                }

                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod test_parser {
    use std::collections::HashMap;

    use crate::http_parser::http_parser::ParserState;

    use super::HTTParser;

    #[test]
    fn test_parsing_response() {
        let response =
            b"HTTP/1.1 200 OK\r\nContent-length: 11\r\nContent-type : plaintext\r\n\r\nHello world";
        let mut parser = HTTParser::new();
        let mut headers_hashmap: HashMap<String, String> = HashMap::new();
        headers_hashmap.insert("content-length".to_string(), "11".to_string());
        headers_hashmap.insert("content-type".to_string(), "plaintext".to_string());
        parser.parse(response);
        assert_eq!(parser.status_code_first_char, '2');
        assert_eq!(parser.headers, headers_hashmap);
    }

    #[test]
    fn test_parsing_resp2() {
        let response = b"HTTP/1.1 200 OK\r\nContent-Length: 11\r\n\r\nHello world";
        let mut parser = HTTParser::new();
        let mut true_headers_map: HashMap<String, String> = HashMap::new();
        true_headers_map.insert("content-length".to_string(), "11".to_string());
        parser.parse(response);
        assert_eq!(true_headers_map, parser.headers);
    }

    #[test]
    fn test_parser_state_on_body() {
        let response = b"HTTP/1.1 200 OK\r\nContent-Length: 11\r\n\r\n";
        let mut parser = HTTParser::new();
        parser.parse(response);
        assert_eq!(parser.state, ParserState::Body)
    }

    #[test]
    fn test_parser_state_on_header() {
        let response = b"HTTP/1.1 200 OK\r\nContent-Length: 11\r\n\r\nHello world";
        let mut parser = HTTParser::new();
        parser.parse(response);
        assert_eq!(parser.state, ParserState::Started);
    }

    #[test]
    fn test_parser_status_code_first_char() {
        let response = b"HTTP/1.1 301\r\n\r\n";
        let mut parser = HTTParser::new();
        parser.parse(response);
        assert_eq!(parser.status_code_first_char, '3');
        assert_eq!(parser.state, ParserState::Started);
    }
    #[test]
    fn test_parser_state_with_empty_body() {
        let response = b"HTTP/1.1 301\r\n\r\n";
        let mut parser = HTTParser::new();
        parser.parse(response);
        assert_eq!(parser.state, ParserState::Started);
    }
}
