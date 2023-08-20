use actix_web::web;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use crate::utils::db::Database;
use crate::utils::encryption::get_digest;
use crate::utils::error::Error;
use crate::utils::general::get_pub_key_path;
use crate::{
    connect::routes::ConnectPeerArgs, utils::communication::send_multicast_msg,
};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hostname::get as get_hostname;
use serde_json::{json, Value};
use std::fs;
use std::sync::Arc;

pub mod echo_helpers {
    use local_ip_address::linux::local_ip;

    #[cfg(target_os = "linux")]
    pub async fn get_local_ip() -> String {
        let ip = local_ip();
        match ip {
            Ok(ip) => ip.to_string(),
            Err(e) => {
                log::error!("Failed to determine local ip: {}", e.to_string());
                "127.0.0.1".to_owned()
            }
        }
    }
    #[cfg(target_os = "android")]
    pub async fn get_local_ip() -> String {
        todo!()
    }
}

pub async fn echo() -> Result<Value, Error> {
    let pub_key_bytes = fs::read(get_pub_key_path().as_str())?;
    let pub_key_encoded_str = URL_SAFE_NO_PAD.encode(pub_key_bytes);
    let hostname = get_hostname()?.to_string_lossy().to_string();
    let local_ip = echo_helpers::get_local_ip().await;

    Ok(
        json!({"pub_key": pub_key_encoded_str, "hostname": hostname, "ip": local_ip}),
    )
}

pub async fn connect_peer(args: &ConnectPeerArgs) -> Result<Value, Error> {
    // Define args
    let pub_key = &args.pub_key;
    let hostname = &args.hostname;
    let ip = &args.ip;

    // Insert data to db
    let mut db = Database::new().await?;
    db.insert_peer(pub_key, hostname, ip).await?;

    Ok(json!({"abuben": "connected"}))
}

mod scan_helpers {
    use serde::Deserialize;
    use serde_json::Value;

    #[derive(Debug, Deserialize)]
    pub struct ScanPeerResponse {
        pub success: bool,
        pub data: Value,
    }
}

pub async fn scan(
    potential_peer_list: web::Data<Arc<Mutex<Vec<String>>>>,
) -> Result<Value, Error> {
    send_multicast_msg(get_digest("resk").await.as_str()).await?;
    sleep(Duration::from_secs(3)).await;

    let mut data = potential_peer_list.lock().await;
    let result = data.clone();
    data.clear();

    Ok(json!({"ip_list": result}))
}
