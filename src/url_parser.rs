use crate::utils;

#[derive(Debug, Clone)]
pub struct ParsedUrl {
    pub host: String,
    pub resource: String,
    pub port: u16,
}

impl ParsedUrl {
    pub fn new(raw_url: &str) -> Result<ParsedUrl, String> {
        let (host, port, resource) = utils::parse_url(raw_url)?;
        Ok(ParsedUrl {
            host,
            port,
            resource,
        })
    }
}
