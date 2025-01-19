use std::env;
use mysql_async::{params, prelude::Queryable, OptsBuilder, Pool};
use uuid::Uuid;

use crate::{api::earthquake::scale_convert, string_to_i64};

fn pool() -> Result<Pool, ()> {
    let mysql_url = OptsBuilder::default()
        .ip_or_hostname(env::var("db_host").unwrap())
        .tcp_port(env::var("db_port").unwrap().parse().unwrap())
        .user(Some(env::var("db_username").unwrap()))
        .pass(Some(env::var("db_passwd").unwrap()))
        .db_name(Some(env::var("db_name").unwrap()));

    let pool = Pool::new(mysql_url);
    return Ok(pool);
}

pub async fn earthquake_save(contents: &Vec<String>) {
    let pool = pool().unwrap();
    let mut conn = pool.get_conn().await.unwrap();
    conn.exec_drop(
        r"INSERT INTO earthquake (id, earthquake_id, type, time, magnitude, depth, intensity, location, tsunami) 
        VALUES (null, :earthquake_id, :type, :time, :magnitude, :depth, :intensity, :location, :tsunami)",
        params! {
            "earthquake_id" => contents.get(0).unwrap_or(&Uuid::new_v4().to_string().replace("-", "")).replace("\"", ""),
            "type" => contents.get(2).unwrap_or(&"".to_string()),
            "time" => contents.get(3).unwrap_or(&"".to_string()).replace("\"", ""),
            "magnitude" => contents.get(11).unwrap_or(&"".to_string()),
            "depth" => string_to_i64(contents.get(10).unwrap_or(&"-1".to_string())).unwrap(),
            "intensity" => scale_convert(string_to_i64(&contents.get(4).unwrap()).unwrap()),
            "location" => contents.get(7).unwrap_or(&"".to_string()).replace("\"", ""),
            "tsunami" => contents.get(7).unwrap_or(&"".to_string()).replace("\"", "")
        }
    ).await.unwrap();
    conn.disconnect().await.unwrap();
    pool.disconnect().await.unwrap();
}

pub async fn earthquake_id_value() -> String {
    let pool = pool().unwrap();
    let mut conn = pool.get_conn().await.unwrap();
    
    let result: Option<String> = conn
        .query_first(
            "SELECT earthquake_id FROM earthquake ORDER BY id DESC LIMIT 1"
        ).await.unwrap();
    
    conn.disconnect().await.unwrap();
    pool.disconnect().await.unwrap();
    
    return result.unwrap_or(Uuid::new_v4().to_string());
}
