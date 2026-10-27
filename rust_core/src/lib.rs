/*
By: Oliver Osmond
Date: 2026-08-31
Program Details: Ironvault Rust WASM backend functions
*/

mod utils;
use crate::utils::database;
use crate::utils::database::DatabaseTable;
use rand::Rng;
use rand::RngCore;
use base64::{engine::general_purpose, Engine as _};
use argon2::{Argon2, Params};
use chacha20poly1305::XChaCha20Poly1305;
use chacha20poly1305::{Key, XNonce};
use chacha20poly1305::aead::{Aead, NewAead};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub async fn login_check(user: String) -> i32 {
    let client = database::create_database_client();
    let id_err_op = client.fetch_id_by_user::<i32>("ironvault", &user).await;
    
    if let Ok(Some(id)) = id_err_op {
        return id;
    }
    return 0;
}

#[wasm_bindgen]
pub async fn create_new_vault(username: String, password_hint: String) -> String {
    let client = database::create_database_client();

    /*
    Here, let id=0 so that our cloudflare worker can auto increment an ID for this entry
    Additionally, the nonce is left empty due to it not being needed for the moment being; it will
    be generated and written later upon re-encryption and updating of the database.
    */
    let vault = DatabaseTable::new(0, username, "".to_string(), generate_salt(16), "".to_string(), password_hint);
    
    match client.insert_record::<DatabaseTable>("ironvault", &vault).await {
        Ok(_) => "".to_string(),
        Err(insert) => format!("{}", insert)
    }
}

#[wasm_bindgen]
pub async fn fetch_db_info(id: i32) -> Vec<String> {
    let client = database::create_database_client();
    
    match client.fetch_record_by_id::<DatabaseTable>("ironvault", id as i64).await {
        Ok(Some(vault)) => vec![
            format!("{}", id), 
            vault.username, 
            vault.ciphertext, 
            vault.kdf_salt, 
            vault.nonce, 
            vault.password_hint
        ],
        Ok(None) => vec!["Record not found".to_string()],
        Err(e) => vec![format!("{:?}", e)]
    }
}

#[wasm_bindgen]
pub fn get_unwrapped_key(password: String, salt: String) -> Vec<u8> {
    match get_key(password, salt) {
        Some(key) => key.to_vec(),
        None => vec![0; 32]
    }
}

#[wasm_bindgen]
pub fn decrypt(nonce: String, key_array: &[u8], ciphertext: String) -> String {
    if ciphertext.is_empty() {
        return "".to_string();
    }
    
    if key_array.len() != 32 {
        return "invalid key length".to_string();
    }

    let Ok(nonce_bytes) = general_purpose::STANDARD.decode(nonce) else {
        return "nonce parsing error".to_string();
    };
    let xnonce = XNonce::from_slice(&nonce_bytes);

    let Ok(ciphertext_bytes) = general_purpose::STANDARD.decode(ciphertext) else {
        return "ciphertext parsing error".to_string();
    };

    let cipher = XChaCha20Poly1305::new(Key::from_slice(key_array));
    
    let Ok(plaintext_bytes) = cipher.decrypt(xnonce, ciphertext_bytes.as_ref()) else {
        return "decryption error".to_string();
    };

    String::from_utf8(plaintext_bytes).unwrap_or_else(|_| "plaintext parsing error".to_string())
}

#[wasm_bindgen]
pub async fn encrypt_and_push(id: i32, plaintext: String, key_array: &[u8]) -> String {
    if key_array.len() != 32 {
        return "invalid key length".to_string();
    }

    let client = database::create_database_client();
    let nonce = generate_nonce();
    let xnonce = XNonce::from_slice(&nonce);
    let nonce_b64 = general_purpose::STANDARD.encode(nonce.clone());

    let cipher = XChaCha20Poly1305::new(Key::from_slice(key_array));

    match cipher.encrypt(xnonce, plaintext.as_bytes()) {
        Ok(ciphertext) => {
            let ciphertext_b64 = general_purpose::STANDARD.encode(ciphertext);

            let perr_one = client.update_record_by_id("ironvault", id as i64, "ciphertext", &serde_json::Value::String(ciphertext_b64)).await;
            let perr_two = client.update_record_by_id("ironvault", id as i64, "nonce", &serde_json::Value::String(nonce_b64)).await;

            if perr_one.is_err() { return "Error in writing ciphertext to database".to_string(); }
            if perr_two.is_err() { return "Error in writing nonce to database".to_string(); }

            "OK".to_string()
        }
        Err(_) => "Encryption error".to_string()
    }
}

pub fn generate_nonce() -> Vec<u8> {
    let mut nonce = [0u8; 24]; 
    rand::thread_rng().fill_bytes(&mut nonce); 
    nonce.to_vec() 
}

pub fn generate_salt(length: usize) -> String {
    let salt_bytes: Vec<u8> = rand::thread_rng() 
        .sample_iter(&rand::distributions::Standard)
        .take(length)
        .collect();
    general_purpose::STANDARD.encode(&salt_bytes)
}

pub fn get_key(mpass: String, salt: String) -> Option<[u8; 32]> {
    let mut key = [0u8; 32]; 
    let params = Params::new(65536, 3, 1, None).expect("Invalid Argon2 parameters"); 
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params); 
    argon2.hash_password_into(mpass.as_bytes(), salt.as_bytes(), &mut key).ok()?;
    Some(key) 
}