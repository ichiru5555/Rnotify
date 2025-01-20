use send_api::{discord_webhook, line_messaging, line_notify};
use api::earthquake;

mod api;
mod send_api;
mod config_load;
mod db;

#[tokio::main]
async fn main() {
    let _ = config_load::load().await;
    earthquake::api().await;
}

pub fn string_to_i64(content: &String) -> Result<i64, std::num::ParseIntError> {
    content.parse::<i64>()
}
