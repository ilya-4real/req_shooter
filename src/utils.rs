use regex::Regex;

pub fn parse_url(url: &str) -> Result<(String, u16, String), String> {
    let re =
        Regex::new(r"(www\.|)([a-zA-Z0-9]+\.[a-z]+|localhost|\d+\.\d+\.\d+\.\d+)(:\d+|)\/(.*)")
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
            Ok((String::from(host), port_num, String::from(resource)))
        }
    }
}

pub fn format_received_data_value(received_data_value: usize) -> String {
    if received_data_value > 1_000_000 {
        return format!("{:.2} MB", received_data_value as f32 / 1_000_000.0);
    }
    if received_data_value > 1_000 {
        format!("{:.2} KB", received_data_value as f32 / 1_000.0)
    } else {
        format!("{} B", received_data_value)
    }
}
