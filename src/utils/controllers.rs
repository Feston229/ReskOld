use crate::connect::controllers::echo_helpers::get_local_ip;
use crate::connect::routes::{connect_peer, echo, scan};
use crate::share::routes::update;
use crate::utils::db::Database;
use crate::utils::error::Error;
use crate::utils::general::{check_keys, get_log_file_path};
use actix_web::{middleware::Logger, App, HttpServer};
use async_once::AsyncOnce;
use clipboard::{ClipboardContext, ClipboardProvider};
use lazy_static::lazy_static;
use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use reqwest::Client;
use serde_json::json;
use std::time::Duration;

use super::encryption::sign_message;

lazy_static! {
    pub static ref DATABASE: AsyncOnce<Database> =
        AsyncOnce::new(async { Database::new().await.unwrap() });
}

pub async fn run() -> Result<(), Error> {
    // Check if files are inplace and init logger
    pre_run().await?;

    // polling to trigger if need to update clipboard of peers
    tokio::spawn(start_pooling_clipboard());

    // Node
    let local_ip = get_local_ip().await;
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .service(connect_peer)
            .service(echo)
            .service(scan)
            .service(update)
    })
    .bind((local_ip.as_str(), 9898))?
    .run()
    .await?;

    Ok(())
}

async fn pre_run() -> Result<(), Error> {
    // CPU heavy tasks
    let pre_run_cpu =
        tokio::task::spawn_blocking(move || -> Result<(), Error> {
            check_keys()?;
            Ok(())
        });
    DATABASE.get().await.apply_migrations().await.unwrap();

    // Logging
    init_logging().await?;

    pre_run_cpu.await??;
    Ok(())
}

#[cfg(target_os = "linux")]
async fn start_pooling_clipboard() {
    let mut clipboard = ClipboardContext::new().unwrap();
    let mut content = clipboard.get_contents().unwrap_or("".to_owned());
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let new_content = clipboard.get_contents().unwrap_or("".to_owned());
        if content != new_content {
            log::info!("new clipboard content -> {}", &new_content);
            content = new_content;
            update_peers(content.clone())
                .await
                .unwrap_or_else(|err| log::error!("{}", err));
        }
    }
}

#[cfg(target_os = "android")]
async fn start_pooling_clipboard() {
    todo!()
}

async fn init_logging() -> Result<(), Error> {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%d %H:%M:%S)} {l} - {m}\n",
        )))
        .build(get_log_file_path())?;
    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))?;
    log4rs::init_config(config)?;
    println!("Logging to {}", get_log_file_path());
    log::info!("Starting Resk node");
    Ok(())
}

async fn update_peers(clipboard: String) -> Result<(), Error> {
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
            let client = Client::builder()
                .timeout(Duration::from_secs(2))
                .build()
                .unwrap();
            let response = client
                .post(&url)
                .body(
                    json!({"clipboard": clipboard, "signature": signature})
                        .to_string(),
                )
                .send()
                .await;
            let response = response.ok();
            match response {
                Some(data) => {
                    log::info!(
                        "{}",
                        format!(
                            "Clipboard shared with {} successfully",
                            &peer.to_owned(),
                        )
                    );
                    println!("{:?}", &data.text().await);
                }
                None => log::info!(
                    "{}",
                    format!(
                        "Failed to share clipboard with peer {}",
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
