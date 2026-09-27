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

//TODO:
//Additionally, functions need to be created that will encrypt and push plaintext to a given user's vault, and
//other utilities for account deletion/modification, etc.

pub async fn create_new_vault(username: String, password_hint: String) -> String{
    let client = database::create_database_client();
    /*
    Here, let id=0 so that our cloudflare worker can auto increment an ID for this entry
    Additionally, the nonce is left empty due to it not being needed for the moment being; it will
    be generated and written later upon re-encryption and updating of the database.
    */
    let vault = DatabaseTable::new(0, username, "".to_string(), generate_salt(16), "".to_string(), password_hint);
    let insert_err = client.insert_record::<DatabaseTable>("ironvault", &vault).await;
    if let Err(insert) = insert_err {
        return format!("{}", insert);  
    }
    return "Created".to_string();
}


/*
    TODO:
    Currently, encrypt_and_push() is in need of the key, and it would be preferable to avoid key derivation and database pulling twice.
    Create a function that returns the user DB entry to JS, then another to pass an optionless derived key, that way we can pass everything
    the fetch_and_decrypt() (Which will just become decrypt) and encrypt_and_push() functions will need to do their job such that info and
    keys never have to be pulled/derived twice. 
*/

pub async fn fetch_and_decrypt(id: i32, password: String) -> String {
    let client = database::create_database_client();
    let vault_op_err = client.fetch_record_by_id::<DatabaseTable>("ironvault", id as i64).await;
    let vault: DatabaseTable;

    if let Ok(vault_op) = vault_op_err {
        if let Some(vault_temp) = vault_op {
            vault = vault_temp;
        } else {
            return format!("{:?}", vault_op);
        }
    } else {
        return format!("{:?}", vault_op_err);
    }

    if vault.ciphertext.is_empty() {
        return "".to_string();
    } else {
        let nonce_bytes_err = general_purpose::STANDARD.decode(vault.nonce);
        let xnonce: &XNonce;
        let Ok(nonce_bytes) = nonce_bytes_err else {
            return "nonce parsing error".to_string();
        };
        xnonce = XNonce::from_slice(&nonce_bytes);


        let key_op = get_key(password, vault.kdf_salt);
        let key: [u8; 32];
        let Some(key_temp) = key_op else {
            return "key parsing error".to_string();
        };
        key = key_temp;

        let ciphertext_err = general_purpose::STANDARD.decode(vault.ciphertext);
        let Ok(ciphertext) = ciphertext_err else {
            return "ciphertext parsing error".to_string();
        };

        let cipher = XChaCha20Poly1305::new(Key::from_slice(&key));
        let plaintext_bytes_err = cipher.decrypt(xnonce, ciphertext.as_ref());
        let Ok(plaintext_bytes) = plaintext_bytes_err else {
            return "decryption error".to_string();
        };

        let plaintext_parsed_err = String::from_utf8(plaintext_bytes);
        let Ok(plaintext) = plaintext_parsed_err else {
            return "plaintext parsing error".to_string();
        };

        return plaintext;
    }
}

pub fn encrypt_and_push(id: i32, plaintext: String, password: String) {

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
