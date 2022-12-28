use std::collections::HashSet;

use reqwest::{header, Result};

pub(super) fn http_get(url: &str) -> Result<String> {
    let mut headers = header::HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json"));

    let client = reqwest::blocking::Client::builder()
         .default_headers(headers)
         .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/87.0.4280.88 Safari/537.36")
         .gzip(true)
         .build()?;
    let response = client.get(url).send()?;

    match response.error_for_status() {
        Ok(resp) => Ok(resp.text()?),
        Err(error) => Err(error),
    }
}

pub(super) fn normalize_pair_with_quotes(symbol: &str, quotes: &HashSet<String>) -> Option<String> {
    for quote in quotes.iter() {
        if symbol.ends_with(quote) {
            let base = symbol.strip_suffix(quote).unwrap();
            return Some(format!("{}/{}", base, quote).to_uppercase());
        }
    }

    None
}
