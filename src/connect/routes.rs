use crate::connect::controllers::{self, echo_helpers::get_local_ip};
use crate::utils::general::{get_remote_ip, Response};
use actix_web::{get, post, web, HttpRequest, Responder};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Deserialize)]
pub struct ConnectPeerArgs {
    pub pub_key: String,
    pub hostname: String,
    pub ip: String,
}

#[post("/connect_peer")]
pub async fn connect_peer(
    req: HttpRequest,
    data: web::Json<ConnectPeerArgs>,
) -> impl Responder {
    // TODO replace it somehow
    if get_remote_ip(&req).await != get_local_ip().await
        && get_remote_ip(&req).await != "127.0.0.1"
    {
        return Response::failure(403, "Forbiden".to_string());
    }

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
pub async fn scan(
    req: HttpRequest,
    potential_peer_list: web::Data<Arc<Mutex<Vec<String>>>>,
) -> impl Responder {
    // TODO replace it somehow
    if get_remote_ip(&req).await != get_local_ip().await
        && get_remote_ip(&req).await != "127.0.0.1"
    {
        return Response::failure(403, "Forbiden".to_string());
    }

    let response = controllers::scan(potential_peer_list).await;
    match response {
        Ok(data) => Response::success(data),
        Err(e) => Response::failure(500, e.to_string()),
    }
}
