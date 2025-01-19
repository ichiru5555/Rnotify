use std::env;

use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;

pub async fn send(title: &str, content: &str) {
    let uuid = Uuid::new_v4();
    let mut headers= HeaderMap::new();
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));
    headers.insert(
        "Authorization",
        HeaderValue::from_str(
            &format!("Bearer {}",
            env::var("line_messaging_channel_access_token").unwrap())
        ).unwrap()
    );
    headers.insert("X-Line-Retry-Key", HeaderValue::from_str(&uuid.to_string()).unwrap());

    let json = json!(
        {
            "to": env::var("line_messaging_message_to").unwrap(),
            "messages":[
                {
                    "type":"text",
                    "text":title
                },
                {
                    "type":"text",
                    "text":content
                }
            ]        
        }
    );

    let client = Client::new();
    let _ = client.post("https://api.line.me/v2/bot/message/push")
        .headers(headers)
        .json(&json)
        .send().await;
}
