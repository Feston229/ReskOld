use crate::share::controllers;
use crate::utils::general::Response;
use actix_web::{post, web, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct UpdateArgs {
    pub clipboard: String,
    pub signature: String,
}

#[post("/update")]
async fn update(data: web::Json<UpdateArgs>) -> impl Responder {
    let args = data.into_inner();
    let response = controllers::update(&args).await;
    match response {
        Ok(data) => Response::success(data),
        Err(e) => Response::failure(500, e.to_string()),
    }
}
