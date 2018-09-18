use std::collections::HashMap;

pub const CALLBACK_HEADER: &str = "Location";

pub fn format_headers(h: &HashMap<String, String>) -> HashMap<String, String> {
    h.iter()
        .map(|(k, v)| (format!("info-{}", k.to_owned()), v.to_owned()))
        .collect()
}

pub fn unformat_headers(h: &HashMap<String, String>) -> HashMap<String, String> {
    h.iter()
        .map(|(k, v)| (k.to_owned().replace("info-{}", ""), v.to_owned()))
        .collect()
}