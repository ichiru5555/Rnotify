use std::env;
use serde_json::Value;
use tokio_tungstenite::connect_async;
use futures_util::StreamExt;
use crate::{db, discord_webhook, line_messaging, line_notify, string_to_i64};

pub async fn api() {
    let (ws_stream, _) = connect_async("wss://api.p2pquake.net/v2/ws").await.unwrap();
    let (_, mut ws_read) = ws_stream.split();

    let receiver_task = tokio::spawn(async move {
        while let Some(msg) = ws_read.next().await {
            match msg {
                Ok(msg) => {
                    let data = msg.into_text().unwrap();
                    if data.len() > 0 {
                        earthquake_data(data.as_str()).await;
                    }
                }
                Err(e) => {
                    eprintln!("{}\n", e);
                    break;
                }
            }
        }
    });

    let shutdown_signal = tokio::spawn(async {
        tokio::signal::ctrl_c().await.unwrap();
    });

    tokio::select! {
        _ = shutdown_signal => {
            println!("プログラムが終了しました\n");
        }
        _ = receiver_task => {
            println!("受信タスクが終了しました\n");
        }
    }
}

async fn earthquake_data(body: &str) {
    let mut contents: Vec<String> = Vec::new();
    
    let parsed_data: Value = match serde_json::from_str(body) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("JSONのパース中にエラーが発生しました: {}", e);
            return;
        }
    };

    let value = if parsed_data.is_array() {
        parsed_data.as_array()
            .and_then(|arr| arr.first())
            .unwrap_or(&parsed_data)
    } else {
        &parsed_data
    };

    if let Some(code) = value.get("code") {
        let code = code.as_i64();
        if code != Some(551) && code != Some(552) && code != Some(554) && code != Some(556) {
            return;
        }
    }

    if let Some(id) = value.get("_id") {
        //地震id
        contents.push(id.to_string());
    }

    if let Some(earthquake_issue) = value.get("issue") {
        if let Some(source) = earthquake_issue.get("source") {
            //発表元
            contents.push(source.to_string());
        }

        if let Some(earthquake_type) = earthquake_issue.get("type") {
            //発表種類
            let earthquake_type = match earthquake_type.as_str() {
                Some("Other") => "その他の情報",
                Some("Foreign") => "遠地地震に関する情報",
                Some("DetailScale") => "各地の震度に関する情報",
                Some("ScaleAndDestination") => "震度・震源に関する情報",
                Some("Destination") => "震源に関する情報",
                Some("ScalePrompt") => "震度速報",
                Some(_) => "不明なデータ",
                None => "データがありません"
            };
            contents.push(earthquake_type.to_string());
        }
    }

    if let Some(earthquake) = value.get("earthquake") {
        if let Some(earthquake_time) = value.get("time") {
            //地震発生時間
            contents.push(earthquake_time.to_string());
        }
            
        if let Some(_max_scale) = earthquake.get("maxScale") {
            //震度

            let discord_color = match _max_scale.as_i64() {
                Some(60..70) => 0xff0000,
                Some(45..55) => 0xffff00,
                Some(10..40) => 0x00008b,
                Some(_) => 0x000000,
                None => 0x000000
            };

            contents.push(_max_scale.to_string());
            contents.push(discord_color.to_string());
        }

        if let Some(_domestic_tsunami) = earthquake.get("domesticTsunami") {
            //津波
            let domestic_tsunami = match _domestic_tsunami.as_str() {
                Some("Warning") => "津波予報(種類不明)",
                Some("Watch") => "津波注意報",
                Some("NonEffective") => "若干の海面変動が予想されるが、被害の心配なし",
                Some("Checking") => "調査中",
                Some("Unknown") => "不明",
                Some("None") => "なし",
                Some(_) => "不明なデータ",
                None => "データがありません"
            };
            contents.push(domestic_tsunami.to_string());
        }

        if let Some(hypocenter) = earthquake.get("hypocenter") {
            if let Some(name) = hypocenter.get("name") {
                //地震発生場所の名前
                contents.push(name.to_string());
            }

            if let Some(latitude) = hypocenter.get("latitude") {
                //緯度
                contents.push(latitude.to_string());
            }

            if let Some(longitude) = hypocenter.get("longitude") {
                //経度
                contents.push(longitude.to_string());
            }

            if let Some(depth) = hypocenter.get("depth") {
                //深さ
                let depth = match depth.as_i64() {
                    Some(0) => "ごく浅い".to_string(),
                    Some(-1) => "存在しない".to_string(),
                    Some(val)  => format!("{}", val),
                    None => "データがありません".to_string()
                };
                contents.push(depth);
            }

            if let Some(magnitude) = hypocenter.get("magnitude") {
                //マグニチュード
                let magnitude = match magnitude.as_f64() {
                    Some(-1.0) => "震源情報が存在しません".to_string(),
                    Some(val) => format!("{:.1}", val),
                    None => "データがありません".to_string()
                };
                contents.push(magnitude);
            }
        }
    }

    if db::earthquake_id_value().await == contents.get(0).unwrap().to_string().replace("\"", "") {
        return;
    }

    let scale = string_to_i64(&contents.get(4).unwrap()).unwrap();

    let title = format!("{:?} {:?}", contents.get(1), contents.get(2))
        .replace("Some(", "")
        .replace(")", "")
        .replace("\\\"", "")
        .replace("\"\"", "")
        .replace("\"", "");

    let content = format!("発表元: {:?}\n発表の種類: {:?}\n地震発生時間: {:?}\n震度: {:?}\n津波: {:?}\n地震発生場所: {:?}\n緯度: {:?}\n経度: {:?}\n震源地の深さ: {:?}\nマグニチュード: {:?}",
        contents.get(1),
        contents.get(2),
        contents.get(3),
        scale_convert(scale),
        contents.get(6),
        contents.get(7),
        contents.get(8),
        contents.get(9),
        contents.get(10),
        contents.get(11)
    ).replace("Some(", "").replace(")", "").replace("\\\"", "").replace("\"", "");

    db::earthquake_save(&contents).await;
    if scale < 30 {
        return;
    }

    if env::var("discord_webhook_url").is_ok() {
        discord_webhook::send(&title, &content, &contents.get(5).unwrap()).await;
    }
    if env::var("line_messaging_channel_access_token").is_ok() && env::var("line_messaging_message_to").is_ok() {
        line_messaging::send(&title, &content).await;
    }
    if env::var("line_notify_token").is_ok() {
        line_notify::send(&content).await;
    }
}

pub fn scale_convert(scale: i64) -> String {
    let scale = match scale {
        70 => "震度7",
        60 => "震度6強",
        55 => "震度6弱",
        50 => "震度5強",
        45 => "震度5弱",
        40 => "震度4",
        30 => "震度3",
        20 => "震度2",
        10 => "震度1",
        -1 => "震度情報なし",
        _ => "不明なデータ"
    };
    return scale.to_string();
}
