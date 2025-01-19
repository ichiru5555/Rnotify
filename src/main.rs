use std::env;
use tokio_cron_scheduler::{JobScheduler, Job};
use send_api::{discord_webhook, line_messaging, line_notify};
use api::earthquake;

mod api;
mod send_api;
mod config_load;
mod db;

#[tokio::main]
async fn main() {
    config_load::load().await;
    let mut scheduler = JobScheduler::new().await.unwrap();

    scheduler.add(
        Job::new_async("0 */5 * * * *", |_uuid, _l: JobScheduler| {
            Box::pin(async move {
                earthquake().await;
            })
        }).unwrap(),
    ).await.unwrap();

    scheduler.start().await.unwrap();

    tokio::signal::ctrl_c().await.unwrap();
    scheduler.shutdown().await.unwrap();
}

async fn earthquake() {
    let res_earthquake_api = earthquake::api().await;
    if let Ok(res) = res_earthquake_api {
        let scale = string_to_i64(&res.get(4).unwrap()).unwrap();

        if db::earthquake_id_value().await == res.get(0).unwrap().to_string().replace("\"", "") {
            print!("同じ地震IDのため処理を停止\n");
            return;
        }
        let title = format!("{:?} {:?}", res.get(1), res.get(2))
            .replace("Some(", "")
            .replace(")", "")
            .replace("\\\"", "")
            .replace("\"\"", "")
            .replace("\"", "");

        let content = format!("発表元: {:?}\n発表の種類: {:?}\n地震発生時間: {:?}\n震度: {:?}\n津波: {:?}\n地震発生場所: {:?}\n緯度: {:?}\n経度: {:?}\n震源地の深さ: {:?}\nマグニチュード: {:?}",
            res.get(1),
            res.get(2),
            res.get(3),
            earthquake::scale_convert(scale),
            res.get(6),
            res.get(7),
            res.get(8),
            res.get(9),
            res.get(10),
            res.get(11)
        ).replace("Some(", "").replace(")", "").replace("\\\"", "").replace("\"", "");
        db::earthquake_save(&res).await;
        if scale > 30 {
            print!("震度3以下のため処理を停止\n");
            return;
        }

        if env::var("discord_webhook_url").is_ok() {
            discord_webhook::send(&title, &content, &res.get(5).unwrap()).await;
        }
        if env::var("line_messaging_channel_access_token").is_ok() && env::var("line_messaging_message_to").is_ok() {
            line_messaging::send(&title, &content).await;
        }
        if env::var("line_notify_token").is_ok() {
            line_notify::send(&content).await;
        }
    } else if let Err(e) = res_earthquake_api {
        print!("{}\n", e);
    }
}

pub fn string_to_i64(content: &String) -> Result<i64, std::num::ParseIntError> {
    content.parse::<i64>()
}
