use reqwest::Client;
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::Value;

pub async fn api() -> Result<Vec<String>, String> {
    let mut contents: Vec<String> = Vec::new();

    let mut headers = HeaderMap::new();
    headers.insert("Accept", HeaderValue::from_static("application/json"));

    let client = Client::new();
    let res = client.get("https://api.p2pquake.net/v2/history?codes=551&limit=1").send().await.unwrap();
    let res = res.text().await;
    if let Ok(body) = res {
        let parsed_data: Vec<Value> = serde_json::from_str(&body).unwrap();
        if let Some(value) = parsed_data.get(0) {
            if let Some(id) = value.get("id") {
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
                        let magnitude = match magnitude.as_i64() {
                            Some(-1) => "震源情報が存在しません".to_string(),
                            Some(val) => format!("{}", val),
                            None => "データがありません".to_string()
                        };
                        contents.push(magnitude);
                    }
                }
            }
        }
        return Ok(contents);
    } else {
        return Err(res.unwrap());
    }
}

pub fn scale_convert(scale: i64) -> String{
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
