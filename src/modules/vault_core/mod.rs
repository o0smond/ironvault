/*
By: Oliver Osmond
Date: 2025-11-30

This file contains the meat of the password manager.
It is very self contained, with the gui simply asking
it to add or remove things to its internal variables
or for it to print out some information. This file also
handles encryption and decryption of the vault, as well
as storage and writing/reading of the vault to the storage.JSON
file also stored in /vault_core
 */
use std::{fs, io::{self}, sync::Mutex, sync::atomic::{AtomicUsize, Ordering}, sync::RwLock};
use serde_json; //for working with the JSON vault
use serde_json::{Value, Map};
use rand::Rng; //for generating random bytes for salts and nonces.
use rand::RngCore;
use base64::{engine::general_purpose, Engine as _};
use argon2::{Argon2, Params}; //uses Argon2 for data encryption/decryption
use chacha20poly1305::XChaCha20Poly1305; //uses chacha20poly1305 as a cypher
use chacha20poly1305::{Key, XNonce};
use chacha20poly1305::aead::{Aead, NewAead};
use once_cell::sync::Lazy;
use zeroize::Zeroize;

/*
This vault self contains all of its data, and simply takes gui input on what to do.
These are all the variables that are used and stored in this vault manager:
*/

static ATTEMPTS: AtomicUsize = AtomicUsize::new(0); //for the error message on the login screen
static PASSWORD: Mutex<Option<String>> = Mutex::new(None); //the master password for key generation
static SPATH: Mutex<Option<String>> = Mutex::new(None); //the path to the storage file, a bit of a leftover from the PyQT version this was adapted from
static KEY: Lazy<RwLock<[u8; 32]>> = Lazy::new(|| RwLock::new([0u8; 32])); //the key from Argon2
static PASSWORD_MAP: Lazy<RwLock<Map<String, Value>>> = Lazy::new(|| { //very important variable holding all of the decrypted data currently being accessed in the vault
    RwLock::new(Map::new())
});


/*
    This function is a more detached one responsable for the backend of the login screen only.
    It takes in a user pasword attempt (mpass) and a storage path and returns either
    "ok" or an error message with an attempt number depending on if the attempted
    decryption of the vault that occurs inside was successful or errored.
*/
pub fn pg1_startup(mpass: &str, storage_path: &str) -> io::Result<String> {
    //Defining static variables
    *PASSWORD.lock().unwrap() = Some(mpass.to_string());
    *SPATH.lock().unwrap() = Some(storage_path.to_string());
    //pulling info from the JSON vault
    let storage_content = fs::read_to_string(storage_path).unwrap_or_else(|_| "{}".to_string());
    let mut vault: serde_json::Value = serde_json::from_str(&storage_content).unwrap_or(serde_json::json!({}));
    
    //if there is cyphertext in the vault, check if it's the correct password, otherwise treat this as a new user with a new master password
    if let Some(existing_data) = vault.get("ciphertext").and_then(|v| v.as_str()) {
        //if it is a returning user. Assumes the JSON has NOT been tampered with.
        //Defining variables for decryption attempt:
        let salt = vault.get("kdf_salt").and_then(|v| v.as_str()).unwrap().to_string();
        let key = get_key(mpass.to_string().clone(), salt.clone()).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        let nonce_b64 = vault.get("nonce").and_then(|v| v.as_str()).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "vault missing nonce"))?;
        let nonce_bytes = general_purpose::STANDARD.decode(nonce_b64).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "bad nonce"))?;
        let xnonce = XNonce::from_slice(&nonce_bytes); //converting b64 stored nonce to usable nonce for XChaCha

        let cipher_bytes = general_purpose::STANDARD.decode(existing_data)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "bad ciphertext"))?;

        let cipher = XChaCha20Poly1305::new(Key::from_slice(&key)); //undoing cypher and attempting decryption of the cyphertext

        //if the decryption works return "ok". Otherwise, return an error message
        if cipher.decrypt(&xnonce, cipher_bytes.as_ref()).is_ok() {
            Ok("ok".to_string())                      // password correct, return "ok"
        } else {
            let attempt = ATTEMPTS.fetch_add(1, Ordering::SeqCst) + 1; //Add 1 to the attemps
            Ok(format!("Wrong Password. This is attempt {}", attempt)) //Retrun error message
        }
    } else {
        //if it's a new user. Generates a salt for later decryptive use.
        let salt = generate_salt(16);
        vault["kdf_salt"] = serde_json::Value::String(salt.clone());
        let json_string = serde_json::to_string_pretty(&vault).unwrap();
        fs::write(storage_path, json_string).unwrap(); //write salt to the JSON
        return Ok("ok".to_string());
    }
}

/*
    This function's purpose is to fully decrept the vault's cyphertext if present and convert it to a map,
    which is then stored in the PASSWORD_MAP variable.
*/
pub fn unlock_vault() -> io::Result<String> {
    //defining variables for decryption
    let storage_path = SPATH.lock().unwrap().as_ref().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "storage path not set"))?.clone();
    let storage_content = fs::read_to_string(&storage_path)
        .unwrap_or_else(|_| "{}".to_string());

    let vault: serde_json::Value =
        serde_json::from_str(&storage_content).unwrap_or_else(|_| serde_json::json!({}));

    let salt = vault["kdf_salt"] //because we know it must exist, granted someone did not have cyphertext and delete their salt
        .as_str()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "vault missing kdf_salt"))?
        .to_string();

    let pass = PASSWORD.lock().unwrap().as_ref().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "password not set"))?.clone();
    let key = get_key(pass.clone(), salt.clone()).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let mut key_guard = KEY.write().unwrap();
    *key_guard = key;
    let return_txt: String;
    //if there is cyphertext in the vault, decrypt it. otherwise set the password map to {}
    if vault.get("ciphertext").and_then(|v| v.as_str()).is_some() {
        let nonce_b64 = vault.get("nonce")
            .and_then(|v| v.as_str())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "vault missing nonce"))?;
        let nonce_bytes = general_purpose::STANDARD
            .decode(nonce_b64)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "bad nonce"))?;
        let xnonce = XNonce::from_slice(&nonce_bytes); //transforming b64 nonce to usable nonce

        //defining plaintext to decrypt
        let plaintext = if let Some(cipher_b64) = vault["ciphertext"].as_str().and_then(|v| Some(v)) {
            //decrypting
            let cipher_bytes = general_purpose::STANDARD.decode(cipher_b64).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "bad ciphertext"))?;
            let cipher = XChaCha20Poly1305::new(Key::from_slice(&key));
            let pt = cipher
                .decrypt(&xnonce, cipher_bytes.as_ref())
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "decryption failed"))?;
            String::from_utf8(pt)
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "plaintext not UTF-8"))?
        } else {
            String::new()
        };
        let p_map: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&plaintext).unwrap_or_else(|_| serde_json::Map::new()); //p_map = password map
        let mut map_guard = PASSWORD_MAP.write().unwrap(); //map_guard to safely access the static variable PASSWORD_MAP
        *map_guard = p_map.clone();
        return_txt = serde_json::to_string_pretty(&p_map).unwrap(); 
    } else {
        return_txt = "{}".to_string();
    }
    Ok(return_txt) //return the password map
}

/*
This function in essence can act as "locking" the vault, but it is more accurately saving the current
contents of the vault to the storage.JSON file. This comes in handy for our "save" feature on the
menu screen.
*/
pub fn lock_vault() -> io::Result<String> {
    let map_guard = PASSWORD_MAP.write().unwrap(); //to access the static safely
    let storage_path = SPATH.lock().unwrap().as_ref().unwrap().clone();
    let storage_content = fs::read_to_string(&storage_path)
        .unwrap_or_else(|_| "{}".to_string());
    let mut vault: serde_json::Value = serde_json::from_str(&storage_content).unwrap_or_else(|_| serde_json::json!({}));
    let key_guard = KEY.read().unwrap();
    let cipher = XChaCha20Poly1305::new(Key::from_slice(&*key_guard)); //Encrypting the plaintext password map
    let new_nonce = generate_nonce();   //generating new nonce
    let new_xnonce = XNonce::from_slice(&new_nonce); //making nonce usable
    vault["nonce"] = serde_json::Value::String(general_purpose::STANDARD.encode(&new_nonce));
    let new_ciphertext = cipher.encrypt(new_xnonce, serde_json::to_string(&*map_guard).unwrap().as_bytes()).unwrap(); //encoding the encypted password map with the new nonce
    let new_ciphertext_b64 = general_purpose::STANDARD.encode(&new_ciphertext); //encoding the cyphertext in b64 for storage
    vault["ciphertext"] = serde_json::Value::String(new_ciphertext_b64);
    let json_string = serde_json::to_string_pretty(&vault).unwrap();
    fs::write(&storage_path, json_string).unwrap(); //writing cyphertext to the vault
    Ok("ok".to_string())
}

//Takes in a username and password and adds them to the decypted password map
pub fn add_password(user: &str, pass: &str) -> io::Result<String> {
    let mut map = PASSWORD_MAP.write().unwrap();
    map.insert(user.to_string(), Value::String(pass.to_string()));
    let return_txt = serde_json::to_string_pretty(&*map).unwrap(); 
    Ok(return_txt)
}

//Retrieves the decrypted password map and returns it
pub fn print_map() -> io::Result<String> {
    let map_guard = PASSWORD_MAP.read().unwrap();
    let return_txt = serde_json::to_string_pretty(&*map_guard).unwrap(); 
    Ok(return_txt)
}

//Removes an entry from the decrypted password map
pub fn delete(user: &str) -> io::Result<()> { 
    let mut map = PASSWORD_MAP.write().unwrap();
    map.remove(user);
    Ok(())
}

//checks if an entry exists in the current decrypted password map
pub fn check_entry(user: &str) -> Result<bool, ()> {
    let map = PASSWORD_MAP.write().unwrap();
    let result = map.contains_key(user);
    Ok(result)
}

//Generates a random nonce for use in the lock_vault() function
pub fn generate_nonce() -> Vec<u8> {
    let mut nonce = [0u8; 24]; //24 byte nonce
    rand::thread_rng().fill_bytes(&mut nonce); //generating random bytes
    nonce.to_vec() //converting to vector for use
}

//Generates a random salt for use in the pg1_startup() function
pub fn generate_salt(length: usize) -> String {
    let salt_bytes: Vec<u8> = rand::thread_rng() //added thing so that you can make it however many bytes long but it will usually be 16
        .sample_iter(&rand::distributions::Standard)
        .take(length)
        .collect();
    return general_purpose::STANDARD.encode(&salt_bytes); //encoding salt in b64 for use and storage
}

//Generates a key using Argon2 with the given master password and salt
pub fn get_key(arg_mpass: String, arg_salt: String) -> io::Result<[u8; 32]> {
    let mut key = [0u8; 32]; //32 byte key
    let params = Params::new(65536, 3, 1, None).expect("Invalid Argon2 parameters"); //defining Argon2 parameters
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params); //initializing Argon2
    argon2.hash_password_into(arg_mpass.as_bytes(), arg_salt.as_bytes(), &mut key) //generating key
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    return Ok(key); //return key
}