use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use ring::rand::SystemRandom;
use ring::signature::{self, Ed25519KeyPair, KeyPair, UnparsedPublicKey};
use std::fs::{self};

use crate::utils::{error::Error, general::get_pub_key_path};

use super::general::get_private_key_path;

pub fn generate_keys() -> Result<(), Error> {
    // Generate keys
    let rng = SystemRandom::new();
    let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng)?;

    // Save private key
    let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())?;
    fs::write(get_private_key_path(), &pkcs8_bytes)?;

    // Save public key
    let pub_key = key_pair.public_key().as_ref();
    fs::write(get_pub_key_path(), pub_key)?;

    Ok(())
}

fn load_private_key() -> Result<Ed25519KeyPair, Error> {
    let private_key_bytes = &fs::read(get_private_key_path())?;
    let private_key = Ed25519KeyPair::from_pkcs8(private_key_bytes)?;
    Ok(private_key)
}

fn load_pub_key(pub_key: &String) -> Result<UnparsedPublicKey<Vec<u8>>, Error> {
    let pub_key = UnparsedPublicKey::new(
        &signature::ED25519,
        URL_SAFE_NO_PAD.decode(pub_key)?,
    );
    Ok(pub_key)
}

pub async fn sign_message(msg: &String) -> Result<String, Error> {
    let private_key = load_private_key()?;
    let sig = private_key.sign(msg.as_bytes());
    let sig_encoded_str = URL_SAFE_NO_PAD.encode(sig.as_ref());
    Ok(sig_encoded_str)
}

pub async fn verify_message(
    pub_key: &String,
    signature: &String,
    msg: &String,
) -> Result<(), Error> {
    let pub_key = load_pub_key(pub_key)?;
    pub_key.verify(msg.as_bytes(), &URL_SAFE_NO_PAD.decode(signature)?)?;
    Ok(())
}
