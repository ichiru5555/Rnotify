use serde_json::Value;
use std::env;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

pub async fn load() {
    let config_file = File::open("config.json").await.unwrap();
    let reader = BufReader::new(config_file);

    let mut contents = String::new();
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await.unwrap() {
        contents.push_str(&line);
    }
    let json: Value = serde_json::from_str(&contents).unwrap();

    if let Some(obj) = json.as_object() {
        for (key, value) in obj {
            if let Some(value_str) = value.as_str() {
                env::set_var(key, value_str);
            }
        }
    }
}
