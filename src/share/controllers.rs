use clipboard::{ClipboardContext, ClipboardProvider};
use serde_json::{json, Value};

use crate::share::routes::UpdateArgs;
use crate::utils::error::Error;

#[cfg(target_os = "linux")]
pub async fn update(args: &UpdateArgs) -> Result<Value, Error> {
    // TODO: verify pub_key

    let mut clipboard =
        ClipboardContext::new().expect("Failed to start clipboard");
    clipboard
        .set_contents(args.clipboard.to_owned())
        .expect("Failed to set clipboard");

    Ok(json!("OK"))
}
