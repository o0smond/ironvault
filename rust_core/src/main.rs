/*
By: Oliver Osmond
Date: 2026-08-31
Program Details: Ironvault Rust WASM backend functions
*/

mod utils;
use crate::utils::database;
use rand::Rng;
use rand::RngCore;
use base64::{engine::general_purpose, Engine as _};
use argon2::{Argon2, Params};
use chacha20poly1305::XChaCha20Poly1305;
use chacha20poly1305::{Key, XNonce};
use chacha20poly1305::aead::{Aead, NewAead};



pub fn main() {

}

pub async fn login_check(user: String) -> i32 {
    let client = database::create_database_client();
    let id_err_op = client.fetch_id_by_user::<i32>("ironvault", &user).await;
    
    if let Ok(id_op) = id_err_op {
        if let Some(id) = id_op {
            return id;
        }
    }
    return 0;
}

//Two megafunctions need to be created now, one creating a new vault (database entry with salt, nonce, password hint, etc.),
//and for people who already have an account that will take username and password and return decrypted cyphertext, or "0" if
//an error occurs/vault empty. 

//Additionally, functions need to be created that will encrypt and push plaintext to a given user's vault, and
//other utilities for account deletion/modification, etc.

pub fn create_new_vault() {

}

pub fn generate_nonce() -> Vec<u8> {
    let mut nonce = [0u8; 24]; //24 byte nonce
    rand::thread_rng().fill_bytes(&mut nonce); //generating random bytes
    nonce.to_vec() //converting to vector for use
}

pub fn generate_salt(length: usize) -> String {
    let salt_bytes: Vec<u8> = rand::thread_rng() //added thing so that you can make it however many bytes long but it will usually be 16
        .sample_iter(&rand::distributions::Standard)
        .take(length)
        .collect();
    return general_purpose::STANDARD.encode(&salt_bytes); //encoding salt in b64 for use and storage
}

//Generates a key using Argon2 with the given master password and salt
pub fn get_key(mpass: String, salt: String) -> Option<[u8; 32]> {
    let mut key = [0u8; 32]; //32 byte key
    let params = Params::new(65536, 3, 1, None).expect("Invalid Argon2 parameters"); //defining Argon2 parameters
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params); //initializing Argon2
    argon2.hash_password_into(mpass.as_bytes(), salt.as_bytes(), &mut key) //generating key
        .ok()?;
    return Some(key); //return key
}
