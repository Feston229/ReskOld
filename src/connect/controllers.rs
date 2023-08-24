use actix_web::web;
use reqwest::Client;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use crate::utils::db::Database;
use crate::utils::encryption::get_digest;
use crate::utils::error::Error;
use crate::utils::general::get_verify_key_path;
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
    let verify_key_bytes = fs::read(get_verify_key_path().as_str())?;
    let verify_key_encoded_str = URL_SAFE_NO_PAD.encode(verify_key_bytes);
    let hostname = get_hostname()?.to_string_lossy().to_string();
    let local_ip = echo_helpers::get_local_ip().await;

    Ok(
        json!({"verify_key": verify_key_encoded_str, "hostname": hostname, "ip": local_ip}),
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
    // Send ping message to the multicast group
    send_multicast_msg(get_digest("resk").await.as_str()).await?;
    sleep(Duration::from_secs(3)).await;

    // lock shared data
    let mut data = potential_peer_list.lock().await;
    let ip_list = data.clone();
    data.clear();

    // send echo to discovered hosts
    let mut handles = Vec::new();
    for ip in ip_list {
        let handle = tokio::spawn(async move {
            let client = Client::builder()
                .timeout(Duration::from_secs(3))
                .build()
                .ok()?;
            let host = format!("http://{}:9898/echo", ip);
            let response = client.get(&host).send().await;
            response.ok()?.text().await.ok()
        });

        handles.push(handle);
    }

    // Parse results
    let mut result = Vec::new();
    for handle in handles {
        if let Ok(Some(data)) = handle.await {
            let response: scan_helpers::ScanPeerResponse =
                serde_json::from_str(&data)?;
            result.push(response.data.clone());
        }
    }

    Ok(json!({"ip_list": result}))
}
