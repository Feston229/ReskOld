use std::path::Path;

use actix_web::{HttpRequest, HttpResponse};
use dirs::home_dir;
use serde::{Deserialize, Serialize};

use super::{encryption::generate_keys, error::Error};

#[derive(Serialize, Deserialize)]
struct SuccessResponse<T> {
    pub success: bool,
    pub data: T,
}

#[derive(Serialize, Deserialize)]
struct FailureResponse {
    pub success: bool,
    pub msg: String,
}

pub struct Response;

impl Response {
    pub fn success<T: Serialize>(data: T) -> HttpResponse {
        HttpResponse::Ok().json(SuccessResponse {
            success: true,
            data,
        })
    }
    pub fn failure(code: u16, msg: String) -> HttpResponse {
        match code {
            400 => HttpResponse::BadRequest().json(FailureResponse {
                success: false,
                msg,
            }),
            403 => HttpResponse::Forbidden().json(FailureResponse {
                success: false,
                msg,
            }),
            500 => HttpResponse::InternalServerError().json(FailureResponse {
                success: false,
                msg,
            }),
            _ => HttpResponse::InternalServerError().json(FailureResponse {
                success: false,
                msg,
            }),
        }
    }
}

pub fn check_keys() -> Result<(), Error> {
    if !Path::new(get_pub_key_path().as_str()).exists()
        || !Path::new(get_private_key_path().as_str()).exists()
    {
        std::fs::create_dir_all(get_keys_dir())?;
        generate_keys()?;
    }
    Ok(())
}

#[cfg(target_os = "linux")]
pub fn get_home_dir() -> String {
    let home_dir = home_dir().unwrap_or("".into());
    home_dir.to_string_lossy().to_string()
}

#[cfg(target_os = "linux")]
pub fn get_keys_dir() -> String {
    format!("{}/.resk/keys", get_home_dir())
}

#[cfg(target_os = "linux")]
pub fn get_pub_key_path() -> String {
    format!("{}/.resk/keys/pub_key.pem", get_home_dir())
}

#[cfg(target_os = "linux")]
pub fn get_private_key_path() -> String {
    format!("{}/.resk/keys/private_key.pem", get_home_dir())
}

#[cfg(target_os = "linux")]
pub fn get_db_path() -> String {
    format!("{}/.resk/resk_db.sqlite", get_home_dir())
}

#[cfg(target_os = "linux")]
pub fn get_log_file_path() -> String {
    format!("{}/.resk/resk.log", get_home_dir())
}

#[cfg(target_os = "android")]
pub fn get_home_dir() -> String {
    todo!()
}

#[cfg(target_os = "android")]
pub fn get_keys_dir() -> String {
    todo!()
}

#[cfg(target_os = "android")]
pub fn get_pub_key_path() -> String {
    todo!()
}

#[cfg(target_os = "android")]
pub fn _get_private_key_path() -> String {
    todo!()
}

#[cfg(target_os = "android")]
pub fn get_db_path() -> String {
    todo!()
}

#[cfg(target_os = "android")]
pub fn get_log_file_path() -> String {
    todo!()
}

pub async fn get_remote_ip(req: &HttpRequest) -> String {
    req.connection_info()
        .realip_remote_addr()
        .unwrap_or("127.0.0.1")
        .to_string()
}
