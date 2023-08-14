use crate::connect::controllers;
use crate::utils::general::Response;
use actix_web::{get, post, web, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ConnectPeerArgs {
    pub pub_key: String,
    pub hostname: String,
    pub ip: String,
}

#[post("/connect_peer")]
pub async fn connect_peer(data: web::Json<ConnectPeerArgs>) -> impl Responder {
    let args = data.into_inner();
    let response = controllers::connect_peer(&args).await;
    match response {
        Ok(data) => Response::success(data),
        Err(e) => Response::failure(500, e.to_string()),
    }
}

#[get("/echo")]
pub async fn echo() -> impl Responder {
    let response = controllers::echo().await;
    match response {
        Ok(data) => Response::success(data),
        Err(e) => Response::failure(500, e.to_string()),
    }
}

#[get("/scan")]
pub async fn scan() -> impl Responder {
    let response = controllers::scan().await;
    match response {
        Ok(data) => Response::success(data),
        Err(e) => Response::failure(500, e.to_string()),
    }
}
