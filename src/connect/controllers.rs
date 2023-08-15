use std::time::Duration;

use crate::utils::db::Database;
use crate::utils::error::Error;
use crate::utils::general::get_pub_key_path;
use crate::{connect::routes::ConnectPeerArgs, utils::general::Response};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use hostname::get as get_hostname;
use reqwest::Client;
use serde_json::{json, Value};
use std::fs;

use self::scan_helpers::ScanPeerResponse;

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

    pub async fn generate_ips(local_ip: String) -> Vec<String> {
        let mut ip_vec = Vec::new();

        for i in 1..=255 {
            let next_octet = i.to_string();
            let ip = format!(
                "{}.{}.{}.{}",
                &local_ip.split(".").nth(0).unwrap(),
                &local_ip.split(".").nth(1).unwrap(),
                &local_ip.split(".").nth(2).unwrap(),
                &next_octet
            );
            if ip != local_ip {
                ip_vec.push(ip);
            }
        }
        ip_vec
    }

    #[derive(Debug, Deserialize)]
    pub struct ScanPeerResponse {
        pub status: bool,
        pub data: Value,
    }
}

pub async fn scan() -> Result<Value, Error> {
    // Define data
    let local_ip = echo_helpers::get_local_ip().await;
    let ip_list = scan_helpers::generate_ips(local_ip).await;
    let mut handles = Vec::new();

    // Iterate through all ips
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
    let mut ip_list = Vec::new();
    for handle in handles {
        if let Ok(Some(data)) = handle.await {
            let response: ScanPeerResponse = serde_json::from_str(&data)?;
            result.push(response.data.clone());
            ip_list.push(response.data.get("ip").unwrap().clone());
        }
    }

    Ok(json!({"ip_list": result}))
}
