use regex::Regex;

#[derive(Debug, Clone)]
pub struct ParsedUrl {
    pub host: String,
    pub resource: String,
    pub port: u16,
}

impl ParsedUrl {
    pub fn parse_url(url: &str) -> Result<ParsedUrl, String> {
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
                let resource = caps.get(4).unwrap().as_str();
                Ok(ParsedUrl {
                    host: host.to_string(),
                    resource: resource.to_string(),
                    port: port_num,
                })
            }
        }
    }
}

#[cfg(test)]
mod test_parsing_url {

    use super::ParsedUrl;

    #[test]
    fn test_resource_parsing() {
        let raw_url = "127.0.0.1:8000/";
        let parsed_url = ParsedUrl::parse_url(&raw_url).unwrap();
        assert_eq!(parsed_url.resource, "/");
    }
    #[test]
    fn test_parsing_empty_resource() {
        let raw_url = "127.0.0.1:8000";
        let parsed = ParsedUrl::parse_url(&raw_url).unwrap();
        assert_eq!(parsed.resource, "");
    }
}
