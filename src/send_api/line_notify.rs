use std::env;

use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};

#[deprecated]
pub async fn send(content: &str) {
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!(
            "Bearer {}",
            env::var("line_notify_token").unwrap()
        )).unwrap(),
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/x-www-form-urlencoded"));

    let clinet = Client::new();
    let _ = clinet.post("https://notify-api.line.me/api/notify")
        .headers(headers)
        .body(format!("message=\n{}", content))
        .send().await;
}
