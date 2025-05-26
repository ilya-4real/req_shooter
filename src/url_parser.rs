use regex::Regex;

use anyhow::{self, bail, Context};

#[derive(Debug, Clone)]
pub struct ParsedUrlAndHeader {
    pub host: String,
    pub resource: String,
    pub port: u16,
    pub header: Option<String>,
}

impl ParsedUrlAndHeader {
    pub fn parse_url(url: &str) -> anyhow::Result<ParsedUrlAndHeader> {
        let re =
            Regex::new(r"(www\.|)([a-zA-Z0-9]+\.[a-z]+|localhost|\d+\.\d+\.\d+\.\d+)(:\d+|)(.*)")?;
        let captures = re.captures(url);
        match captures {
            None => bail!("Invald url format use format like: www.example.com:1234"),
            Some(caps) => {
                let host = caps
                    .get(2)
                    .ok_or(anyhow::anyhow!("Unable to parse hostname"))?
                    .as_str();
                let mut port_str = caps
                    .get(3)
                    .ok_or(anyhow::anyhow!("Unable to parse port number"))?
                    .as_str();
                let resource = caps
                    .get(4)
                    .ok_or(anyhow::anyhow!("Unable to parse resource"))?
                    .as_str();
                if port_str.len() <= 1 {
                    port_str = " ";
                };
                let port_num = match port_str[1..].parse::<u16>() {
                    Ok(port) => port,
                    Err(_) => {
                        println!("Unable to parse port number using default 80 port");
                        80
                    }
                };
                let resource = match resource {
                    "" => "/",
                    res => res,
                };
                Ok(ParsedUrlAndHeader {
                    host: host.to_string(),
                    resource: resource.to_string(),
                    port: port_num,
                    header: None,
                })
            }
        }
    }
    pub fn add_header(&mut self, header: String) -> anyhow::Result<()> {
        let header_regex = Regex::new(r"^[a-zA-Z0-9-_ ]+: .*$")
            .context("Internal error: unable to create header regex")?;
        match header_regex.is_match(&header) {
            true => {
                self.header = Some(header);
                Ok(())
            }
            false => bail!("header contain some error"),
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

    use super::ParsedUrlAndHeader;

    #[test]
    fn test_resource_parsing() {
        let raw_url = "127.0.0.1:8000/";
        let parsed_url = ParsedUrlAndHeader::parse_url(&raw_url).unwrap();
        assert_eq!(parsed_url.resource, "/");
    }
    #[test]
    fn test_parsing_empty_resource() {
        let raw_url = "127.0.0.1:8000";
        let parsed = ParsedUrlAndHeader::parse_url(&raw_url).unwrap();
        assert_eq!(parsed.resource, "/");
    }

    #[test]
    fn test_adding_header() {
        let raw_header = "x-Custom-Header: any value you want".to_string();
        let url = "127.0.0.1:8000";
        let mut parsed_url = ParsedUrlAndHeader::parse_url(url).unwrap();
        parsed_url.add_header(raw_header.clone()).unwrap();
        assert_eq!(Some(raw_header), parsed_url.header);
    }

    #[test]
    #[should_panic]
    fn test_adding_bad_header() {
        let raw_header = "x-Custom-He@der: any value you want".to_string();
        let url = "127.0.0.1:8000";
        let mut parsed_url = ParsedUrlAndHeader::parse_url(url).unwrap();
        parsed_url.add_header(raw_header.clone()).unwrap();
    }

    #[test]
    fn test_compiling_request_without_header() {
        let url = "127.0.0.1:8000/resource";
        let mut parsed_url = ParsedUrlAndHeader::parse_url(url).unwrap();
        assert_eq!(
            "GET /resource HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
            parsed_url.compile_request()
        );
    }
    #[test]
    fn test_invalid_port() {
        let url = "127.0.0.1:invalid";
        let parsed_url = ParsedUrlAndHeader::parse_url(url).unwrap();
        assert_eq!(parsed_url.port, 80);
    }
    #[test]
    fn test_empty_port() {
        let url = "localhost";
        let parsed_url = ParsedUrlAndHeader::parse_url(url).unwrap();
        assert_eq!(parsed_url.port, 80);
    }
}
