use crate::utils::db::Database;
use crate::utils::general::Response;
use crate::{share::controllers, utils::general::get_remote_ip};
use actix_web::{post, web, HttpRequest, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct UpdateArgs {
    pub clipboard: String,
    pub signature: String,
}

#[post("/update")]
async fn update(
    req: HttpRequest,
    data: web::Json<UpdateArgs>,
) -> impl Responder {
    // TODO replace it somehow
    let db = Database::new().await.unwrap();
    let peer_ip_list = db.get_peers_ip().await.unwrap_or(vec![]);
    if !peer_ip_list.contains(&get_remote_ip(&req).await) {
        return Response::failure(403, "Forbiden".to_string());
    }

    let args = data.into_inner();
    let response = controllers::update(&args).await;
    match response {
        Ok(data) => Response::success(data),
        Err(e) => Response::failure(500, e.to_string()),
    }
}
