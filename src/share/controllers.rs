use clipboard::{ClipboardContext, ClipboardProvider};
use serde_json::{json, Value};

use crate::share::routes::UpdateArgs;
use crate::utils::db::Database;
use crate::utils::encryption::verify_message;
use crate::utils::error::Error;

pub async fn update(args: &UpdateArgs) -> Result<Value, Error> {
    let mut response = "OK";

    let db = Database::new().await?;
    let peer_pub_key = db
        .get_peer_pub_key(&args.remote_ip.as_ref().unwrap())
        .await?
        .unwrap();
    let result =
        verify_message(&peer_pub_key, &args.signature, &args.clipboard).await;

    if result.is_err() {
        response = "Failed to verify signature";
    }

    if result.is_ok() {
        set_clipboard(&args.clipboard).await?;
    }

    Ok(json!(response))
}

#[cfg(target_os = "linux")]
async fn set_clipboard(content: &String) -> Result<(), Error> {
    let clipboard = ClipboardContext::new();
    if clipboard.is_err() {
        return Err(Error::Generic("Failed to init clipboard".into()));
    }
    let mut clipboard = clipboard.unwrap();

    let res = clipboard.set_contents(content.to_owned());
    if res.is_err() {
        return Err(Error::Generic("Failed to set clipboard".into()));
    }
    Ok(())
}
