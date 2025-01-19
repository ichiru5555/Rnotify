use std::env;

use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde_json::json;

pub async fn send(title: &str, content: &str, color: &str) {
    if env::var("discord_webhook_url").is_err() {
        return;
    }
    let data = json!(
        {
            "embeds": [
                {
                    "title": title,
                    "description": content,
                    "color": color
                }
            ]
        }
    );
    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/json"));
    let client = Client::new();
    let _ = client.post(env::var("discord_webhook_url").unwrap())
        .headers(headers)
        .json(&data)
        .send().await;
}
