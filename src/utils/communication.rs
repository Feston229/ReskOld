use super::controllers::SOCKET;

use super::encryption::get_digest;
use super::encryption::sign_message;
use crate::connect::controllers::echo_helpers::get_local_ip;
use crate::utils::{db::Database, error::Error};
use reqwest::Client;
use serde_json::json;
use std::{
    net::{Ipv4Addr, SocketAddrV4},
    str::FromStr,
    sync::Arc,
};
use tokio::{sync::Mutex, time::Duration};

pub async fn update_peers(clipboard: String) -> Result<(), Error> {
    // Define data
    let db = Database::new().await?;
    let peers_ip = db.get_peers_ip().await?;
    let mut handles = Vec::new();
    let signature = sign_message(&clipboard).await?;

    // Iterate through all peers
    for peer in peers_ip {
        let signature = signature.clone();
        let clipboard = clipboard.clone();
        let handle = tokio::spawn(async move {
            let url = format!("http://{}:9898/update", &peer);
            let body = json!({"clipboard": clipboard, "signature": signature});
            let client = Client::builder()
                .timeout(Duration::from_secs(2))
                .build()
                .unwrap();
            let response = client
                .post(&url)
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(body.to_string())
                .send()
                .await;
            let response = response.ok();
            match response {
                Some(data) => {
                    let res = data.text().await.unwrap();
                    let tost =
                        serde_json::Value::from_str(&res.clone()).unwrap();
                    let dat = tost.get("data").unwrap().clone();
                    if dat.to_string() == "\"OK\"".to_string() {
                        log::info!(
                            "{}",
                            format!(
                                "Clipboard shared with {} successfully",
                                &peer.to_owned(),
                            )
                        );
                    } else {
                        log::info!(
                            "{}",
                            format!(
                                "Failed to share clipboard with peer {}: {}",
                                &peer.to_owned(),
                                res
                            )
                        );
                    }
                }
                None => log::info!(
                    "{}",
                    format!(
                        "Failed to share clipboard with peer {}, no response",
                        &peer.to_owned(),
                    )
                ),
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap_or_else(|err| {
            log::error!("{}", format!("tokio error: {}", err))
        });
    }

    Ok(())
}

pub async fn start_broadcasting(potential_peer_list: Arc<Mutex<Vec<String>>>) {
    let multicast_addr: Ipv4Addr = "239.0.0.1".parse().unwrap();
    let port: u16 = 23235;

    //tokio::spawn(multicast_client(multicast_addr, port));
    tokio::spawn(multicast_server(potential_peer_list));
}
// Used to send ping to other peers
pub async fn multicast_client(multicast_addr: Ipv4Addr, port: u16) {
    let socket = SOCKET.get().await;
    loop {
        // Send ping message
        let ping_msg = "Ping";
        socket
            .send_to(
                ping_msg.as_bytes(),
                SocketAddrV4::new(multicast_addr, port),
            )
            .await
            .expect("BAN");

        // Wait before sending the next ping
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

// Used to receive that ping
pub async fn multicast_server(potential_peer_list: Arc<Mutex<Vec<String>>>) {
    let mut buf = [0u8; 4096];
    let socket = SOCKET.get().await;

    loop {
        if let Ok((size, addr)) = socket.recv_from(&mut buf).await {
            // Skip requests from localhost
            if addr.ip().to_string() == get_local_ip().await {
                continue;
            }

            let response = String::from_utf8_lossy(&buf[..size]);

            if response == get_digest("resk").await.as_str() {
                socket
                    .send_to(get_digest("yes").await.as_bytes(), addr)
                    .await
                    .unwrap_or_else(|err| {
                        log::error!("Error pinging back: {}", err);
                        0 as usize
                    });
                log::info!("Ping sent back to: {}", addr.ip());
            }
            if response == get_digest("yes").await.as_str() {
                let mut vec = potential_peer_list.lock().await;
                if !vec.contains(&addr.ip().to_string()) {
                    vec.push(addr.ip().to_string());
                    log::info!("Discovered host: {}", addr.ip());
                }
            }
        }
    }
}

pub async fn send_multicast_msg(msg: &str) -> Result<(), Error> {
    let socket = SOCKET.get().await;
    let multicast_addr: Ipv4Addr = "239.0.0.1".parse().unwrap();
    let port: u16 = 23235;

    let dest = SocketAddrV4::new(multicast_addr, port);

    socket.send_to(msg.as_bytes(), dest).await?;
    log::info!("Sent message {} to multicast group", msg);
    Ok(())
}
