use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use ring::digest;
use ring::rand::SystemRandom;
use ring::signature::{self, Ed25519KeyPair, KeyPair, UnparsedPublicKey};
use std::fs;

use crate::utils::{error::Error, general::get_verify_key_path};

use super::general::get_sign_key_path;

pub fn generate_keys() -> Result<(), Error> {
    // Generate keys
    let rng = SystemRandom::new();
    let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng)?;

    // Save private key
    let sign_key = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())?;
    fs::write(get_sign_key_path(), &pkcs8_bytes)?;

    // Save public key
    let verify_key = sign_key.public_key().as_ref();
    fs::write(get_verify_key_path(), verify_key)?;

    Ok(())
}

fn load_sign_key() -> Result<Ed25519KeyPair, Error> {
    let sign_key_bytes = &fs::read(get_sign_key_path())?;
    let sign_key = Ed25519KeyPair::from_pkcs8(&sign_key_bytes)?;
    Ok(sign_key)
}

fn load_verify_key(
    verify_key: &String,
) -> Result<UnparsedPublicKey<Vec<u8>>, Error> {
    let verify_key = UnparsedPublicKey::new(
        &signature::ED25519,
        URL_SAFE_NO_PAD.decode(verify_key)?,
    );
    Ok(verify_key)
}

pub async fn sign_message(msg: &String) -> Result<String, Error> {
    let sign_key = load_sign_key()?;
    let sig = sign_key.sign(msg.as_bytes());
    let sig_encoded_str = URL_SAFE_NO_PAD.encode(sig.as_ref());
    Ok(sig_encoded_str)
}

pub async fn verify_message(
    verify_key: &String,
    signature: &String,
    msg: &String,
) -> Result<(), Error> {
    let verify_key = load_verify_key(verify_key)?;
    verify_key.verify(msg.as_bytes(), &URL_SAFE_NO_PAD.decode(signature)?)?;
    Ok(())
}

pub async fn get_digest(msg: &str) -> String {
    digest::digest(&digest::SHA512, msg.as_bytes())
        .as_ref()
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect::<String>()
}
