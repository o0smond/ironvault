/*
By: Oliver Osmond
Date: 2025-11-30
Program Details: Rust logic. Contains all functions that are called from Python regarding vault managment.
 */
use pyo3::prelude::*;
use std::{fs, io::{self, Write}, sync::Mutex, sync::atomic::{AtomicUsize, Ordering}, sync::RwLock};
use serde::{Serialize, Deserialize};
use serde_json;
use serde_json::{Value, Map};
use rand::Rng;
use rand::RngCore;
use base64::{engine::general_purpose, Engine as _};
use argon2::{Argon2, Params};
use chacha20poly1305::XChaCha20Poly1305;
use chacha20poly1305::{Key, XNonce};
use chacha20poly1305::aead::{Aead, NewAead};
use pyo3::exceptions::PyValueError;
use once_cell::sync::Lazy;


static ATTEMPTS: AtomicUsize = AtomicUsize::new(0);
static PASSWORD: Mutex<Option<String>> = Mutex::new(None);
static SPATH: Mutex<Option<String>> = Mutex::new(None);
static KEY: Lazy<RwLock<[u8; 32]>> = Lazy::new(|| RwLock::new([0u8; 32]));
static PASSWORD_MAP: Lazy<RwLock<Map<String, Value>>> = Lazy::new(|| {
    RwLock::new(Map::new())
});

#[pyfunction]
fn pg1_startup(mpass: &str, storage_path: &str) -> PyResult<String> {
    *PASSWORD.lock().unwrap() = Some(mpass.to_string());
    *SPATH.lock().unwrap() = Some(storage_path.to_string());
    let storage_content = fs::read_to_string(storage_path).unwrap_or_else(|_| "{}".to_string());
    let mut vault: serde_json::Value = serde_json::from_str(&storage_content).unwrap_or(serde_json::json!({}));
    
    if let Some(existing_data) = vault.get("ciphertext").and_then(|v| v.as_str()) {
        let salt = vault.get("kdf_salt").and_then(|v| v.as_str()).unwrap().to_string();
        let key = get_key(mpass.to_string().clone(), salt.clone())?;
        let nonce_b64 = vault.get("nonce").and_then(|v| v.as_str()).ok_or_else(|| PyValueError::new_err("vault missing nonce"))?;
        let nonce_bytes = general_purpose::STANDARD.decode(nonce_b64).map_err(|_| PyValueError::new_err("bad nonce"))?;
        let xnonce = XNonce::from_slice(&nonce_bytes);

        let cipher_bytes = base64::decode(existing_data)
        .map_err(|_| PyValueError::new_err("bad ciphertext"))?;

        let cipher = XChaCha20Poly1305::new(Key::from_slice(&key));

        if cipher.decrypt(&xnonce, cipher_bytes.as_ref()).is_ok() {
            Ok("ok".to_string())                      // password correct
        } else {
            let attempt = ATTEMPTS.fetch_add(1, Ordering::SeqCst) + 1;
            Ok(format!("Wrong Password. This is attempt {}", attempt))
        }
    } else {
        let salt = generate_salt(16);
        vault["kdf_salt"] = serde_json::Value::String(salt.clone());
        let json_string = serde_json::to_string_pretty(&vault).unwrap();
        fs::write(storage_path, json_string).unwrap();
        return Ok("ok".to_string());
    }
}

#[pyfunction]
fn unlock_vault() -> PyResult<String> {
    let storage_path = SPATH.lock().unwrap().as_ref().ok_or_else(|| PyValueError::new_err("storage path not set"))?.clone();
    let storage_content = fs::read_to_string(&storage_path)
        .unwrap_or_else(|_| "{}".to_string());

    let mut vault: serde_json::Value =
        serde_json::from_str(&storage_content).unwrap_or_else(|_| serde_json::json!({}));

    let salt = vault["kdf_salt"]
        .as_str()
        .ok_or_else(|| PyValueError::new_err("vault missing kdf_salt"))?
        .to_string();

    let pass = PASSWORD.lock().unwrap().as_ref().ok_or_else(|| PyValueError::new_err("password not set"))?.clone();
    let key = get_key(pass.clone(), salt.clone())?;

    let mut key_guard = KEY.write().unwrap();
    *key_guard = key;

    let mut return_txt = serde_json::to_string_pretty(&*PASSWORD_MAP.read().unwrap()).unwrap();
    if let Some(existing_data) = vault.get("ciphertext").and_then(|v| v.as_str()) {
        let nonce_b64 = vault.get("nonce")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PyValueError::new_err("vault missing nonce"))?;
        let nonce_bytes = general_purpose::STANDARD
            .decode(nonce_b64)
            .map_err(|_| PyValueError::new_err("bad nonce"))?;
        let xnonce = XNonce::from_slice(&nonce_bytes);

        let plaintext = if let Some(cipher_b64) = vault["ciphertext"].as_str().and_then(|v| Some(v)) {
            let cipher_bytes = base64::decode(cipher_b64).map_err(|_| PyValueError::new_err("bad ciphertext"))?;

            let cipher = XChaCha20Poly1305::new(Key::from_slice(&key));
            let pt = cipher
                .decrypt(&xnonce, cipher_bytes.as_ref())
                .map_err(|_| PyValueError::new_err("decryption failed"))?;
            String::from_utf8(pt)
                .map_err(|_| PyValueError::new_err("plaintext not UTF-8"))?
        } else {
            String::new()
        };
        let mut p_map: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&plaintext).unwrap_or_else(|_| serde_json::Map::new());
        let mut map_guard = PASSWORD_MAP.write().unwrap();
        *map_guard = p_map.clone();
        return_txt = serde_json::to_string_pretty(&p_map).unwrap(); 
    } else {
        return_txt = "{}".to_string();
    }
    Ok(return_txt)
}

#[pyfunction]
fn lock_vault() -> PyResult<String> {
    let mut map_guard = PASSWORD_MAP.write().unwrap();
    let storage_path = SPATH.lock().unwrap().as_ref().unwrap().clone();
    let storage_content = fs::read_to_string(&storage_path)
        .unwrap_or_else(|_| "{}".to_string());
    let mut vault: serde_json::Value =
        serde_json::from_str(&storage_content).unwrap_or_else(|_| serde_json::json!({}));
    let key_guard = KEY.read().unwrap();
    let cipher = XChaCha20Poly1305::new(Key::from_slice(&*key_guard));
    let new_nonce = generate_nonce();
    let new_xnonce = XNonce::from_slice(&new_nonce);
    vault["nonce"] = serde_json::Value::String(base64::encode(&new_nonce));
    let new_ciphertext = cipher.encrypt(new_xnonce, serde_json::to_string(&*map_guard).unwrap().as_bytes()).unwrap();
    let new_ciphertext_b64 = base64::encode(&new_ciphertext);
    vault["ciphertext"] = serde_json::Value::String(new_ciphertext_b64);
    let json_string = serde_json::to_string_pretty(&vault).unwrap();
    fs::write(&storage_path, json_string).unwrap();
    Ok("ok".to_string())
}

#[pyfunction]
fn add_password(user: &str, pass: &str) -> PyResult<String> {
    let mut map = PASSWORD_MAP.write().unwrap();
    map.insert(user.to_string(), Value::String(pass.to_string()));
    let return_txt = serde_json::to_string_pretty(&*map).unwrap(); 
    Ok(return_txt)
}

#[pyfunction]
fn print_map() -> PyResult<String> {
    let map_guard = PASSWORD_MAP.read().unwrap();
    let return_txt = serde_json::to_string_pretty(&*map_guard).unwrap(); 
    Ok(return_txt)
}

#[pyfunction]
fn delete(user: &str) -> PyResult<()> { 
    let mut map = PASSWORD_MAP.write().unwrap();
    map.remove(user);
    let return_txt = serde_json::to_string_pretty(&*map).unwrap();
    Ok(())
}

#[pymodule]
fn vault_core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(pg1_startup, m)?)?;
    m.add_function(wrap_pyfunction!(unlock_vault, m)?)?;
    m.add_function(wrap_pyfunction!(lock_vault, m)?)?;
    m.add_function(wrap_pyfunction!(add_password, m)?)?;
    m.add_function(wrap_pyfunction!(print_map, m)?)?;
    m.add_function(wrap_pyfunction!(delete, m)?)?;
    Ok(())
}

fn generate_nonce() -> Vec<u8> {
    let mut nonce = [0u8; 24];
    rand::thread_rng().fill_bytes(&mut nonce);
    nonce.to_vec()
}

 fn generate_salt(length: usize) -> String {
    let salt_bytes: Vec<u8> = rand::thread_rng()
        .sample_iter(&rand::distributions::Standard)
        .take(length)
        .collect();
    return general_purpose::STANDARD.encode(&salt_bytes);
}

fn get_key(arg_mpass: String, arg_salt: String) -> Result<[u8; 32], PyErr> {
    let mut key = [0u8; 32];
    let params = Params::new(65536, 3, 1, None).expect("Invalid Argon2 parameters");
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    argon2.hash_password_into(arg_mpass.as_bytes(), arg_salt.as_bytes(), &mut key).or_else(|_| Err(PyValueError::new_err("decryption failed")))?;
    return Ok(key);
}