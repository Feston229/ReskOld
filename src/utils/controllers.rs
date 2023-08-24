use crate::share::routes::update;
use crate::utils::{
    db::Database,
    error::Error,
    general::{check_keys, get_log_file_path},
};
use crate::{
    connect::{
        controllers::echo_helpers::get_local_ip,
        routes::{connect_peer, echo, scan},
    },
    utils::general::get_db_path,
};
use actix_web::{middleware::Logger, web, App, HttpServer};
use async_once::AsyncOnce;
use clipboard::{ClipboardContext, ClipboardProvider};
use lazy_static::lazy_static;
use log::LevelFilter;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};
use std::{
    net::{Ipv4Addr, SocketAddrV4},
    path::Path,
    sync::Arc,
};
use tokio::fs::OpenOptions;
use tokio::{net::UdpSocket as TokioUdpSocket, sync::Mutex, time::Duration};

use super::communication::{start_broadcasting, update_peers};

lazy_static! {
    pub static ref DATABASE: AsyncOnce<Database> =
        AsyncOnce::new(async { Database::new().await.unwrap() });
    pub static ref SOCKET: AsyncOnce<TokioUdpSocket> =
        AsyncOnce::new(async { init_socket().await });
}

pub async fn run() -> Result<(), Error> {
    // Define data
    let potential_peer_list: Arc<Mutex<Vec<String>>> =
        Arc::new(Mutex::new(vec![]));

    // Check if files are inplace and init logger
    pre_run().await?;

    // polling to trigger if need to update clipboard of peers
    tokio::spawn(start_pooling_clipboard());

    // polling to update peer's addresses between each other
    tokio::spawn(start_broadcasting(potential_peer_list.clone()));

    // Node
    let local_ip = get_local_ip().await;
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::clone(&web::Data::new(
                potential_peer_list.clone(),
            )))
            .service(connect_peer)
            .service(echo)
            .service(scan)
            .service(update)
    })
    .bind(("0.0.0.0", 9898))?
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

    // database init
    check_db().await?;

    // Logging
    init_logging().await?;

    pre_run_cpu.await??;

    Ok(())
}

async fn check_db() -> Result<(), Error> {
    if !Path::new(get_db_path().as_str()).exists() {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(get_db_path().as_str())
            .await?;
    }
    DATABASE.get().await.apply_migrations().await?;
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

async fn init_socket() -> TokioUdpSocket {
    let multicast_addr: Ipv4Addr = "239.0.0.1".parse().unwrap();
    let local_addr: Ipv4Addr = "0.0.0.0".parse().unwrap();
    let port: u16 = 23235;

    let tokio_socket =
        TokioUdpSocket::bind(SocketAddrV4::new(local_addr, port))
            .await
            .expect("Failed to bind Tokio socket");
    tokio_socket
        .join_multicast_v4(multicast_addr.clone(), local_addr.clone())
        .expect("Failed to join multicast group for Tokio socket");
    tokio_socket
}
