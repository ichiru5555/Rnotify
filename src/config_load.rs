use serde_json::Value;
use std::{env, io};
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

pub async fn load() -> Result<(), io::Error> {
    let config_file = File::open("config.json").await?;
    let reader = BufReader::new(config_file);

    let mut contents = String::new();
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await? {
        contents.push_str(&line);
    }
    let json: Value = serde_json::from_str(&contents)?;

    if let Some(obj) = json.as_object() {
        for (key, value) in obj {
            if let Some(value_str) = value.as_str() {
                env::set_var(key, value_str);
            }
        }
    }
    Ok(())
}
