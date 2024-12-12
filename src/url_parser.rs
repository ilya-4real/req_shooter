use regex::Regex;

#[derive(Debug, Clone)]
pub struct ParsedUrlHeader {
    pub host: String,
    pub resource: String,
    pub port: u16,
    pub header: Option<String>,
}

impl ParsedUrlHeader {
    pub fn parse_url(url: &str) -> Result<ParsedUrlHeader, String> {
        let re =
            Regex::new(r"(www\.|)([a-zA-Z0-9]+\.[a-z]+|localhost|\d+\.\d+\.\d+\.\d+)(:\d+|)(.*)")
                .unwrap();
        let caps = re.captures(url);
        match caps {
            None => Err("Match not found".to_string()),
            Some(caps) => {
                let host = caps.get(2).unwrap().as_str();
                let port_str = caps.get(3).unwrap().as_str();
                let port_num: u16;
                if port_str.len() > 1 {
                    port_num = port_str[1..].parse::<u16>().unwrap();
                } else {
                    port_num = 80;
                }
                let resource = match caps.get(4).unwrap().as_str() {
                    "" => "/",
                    res => res,
                };
                Ok(ParsedUrlHeader {
                    host: host.to_string(),
                    resource: resource.to_string(),
                    port: port_num,
                    header: None,
                })
            }
        }
    }
    pub fn add_header(&mut self, header: String) -> Result<(), String> {
        let header_regex =
            Regex::new(r"^[a-zA-Z0-9-_ ]+: .*$").expect("unable to create headers regex");
        match header_regex.is_match(&header) {
            true => {
                self.header = Some(header);
                Ok(())
            }
            false => Err("header contain some error".to_string()),
        }
    }

    pub fn compile_request(&mut self) -> String {
        match &self.header {
            Some(header) => {
                format!(
                    "GET {} HTTP/1.1\r\nHost: {}\r\n{}\r\n\r\n",
                    self.resource, self.host, header
                )
            }
            None => {
                format!(
                    "GET {} HTTP/1.1\r\nHost: {}\r\n\r\n",
                    self.resource, self.host
                )
            }
        }
    }
}

#[cfg(test)]
mod test_parsing_url {

    use super::ParsedUrlHeader;

    #[test]
    fn test_resource_parsing() {
        let raw_url = "127.0.0.1:8000/";
        let parsed_url = ParsedUrlHeader::parse_url(&raw_url).unwrap();
        assert_eq!(parsed_url.resource, "/");
    }
    #[test]
    fn test_parsing_empty_resource() {
        let raw_url = "127.0.0.1:8000";
        let parsed = ParsedUrlHeader::parse_url(&raw_url).unwrap();
        assert_eq!(parsed.resource, "/");
    }

    #[test]
    fn test_adding_header() {
        let raw_header = "x-Custom-Header: any value you want".to_string();
        let url = "127.0.0.1:8000";
        let mut parsed_url = ParsedUrlHeader::parse_url(url).unwrap();
        parsed_url.add_header(raw_header.clone()).unwrap();
        assert_eq!(Some(raw_header), parsed_url.header);
    }

    #[test]
    #[should_panic]
    fn test_adding_bad_header() {
        let raw_header = "x-Custom-He@der: any value you want".to_string();
        let url = "127.0.0.1:8000";
        let mut parsed_url = ParsedUrlHeader::parse_url(url).unwrap();
        parsed_url.add_header(raw_header.clone()).unwrap();
    }

    #[test]
    fn test_compiling_request_without_header() {
        let url = "127.0.0.1:8000/resource";
        let mut parsed_url = ParsedUrlHeader::parse_url(url).unwrap();
        assert_eq!(
            "GET /resource HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
            parsed_url.compile_request()
        );
    }
}
