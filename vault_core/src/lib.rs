use pyo3::prelude::*;
use std::{fs, io::{self, Write}};
use serde::{Serialize, Deserialize};
use serde_json;
use rand::Rng;
use rand::RngCore;
use base64::{engine::general_purpose, Engine as _};
use argon2::{Argon2, Params};
use chacha20poly1305::XChaCha20Poly1305;
use chacha20poly1305::{Key, XNonce};
use chacha20poly1305::aead::{Aead, NewAead};
use pyo3::exceptions::PyValueError;

const VAULT_PATH: &str = "storage.json";   
static mut ATTEMPTS: u32 = 0; 

#[pyfunction]
fn pg1_startup(mpass: &str) -> PyResult<String> {
    let storage_content = fs::read_to_string(VAULT_PATH)
        .unwrap_or_else(|_| "{}".to_string());
    let vault: serde_json::Value = serde_json::from_str(&storage_content).map_err(|e| PyValueError::new_err(e.to_string()))?;

    if vault.get("ciphertext").is_none() {
        return Ok("ok".into());
    }

    let salt_b64 = vault["kdf_salt"].as_str()
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("vault missing kdf_salt"))?;
    let nonce_b64 = vault["nonce"].as_str()
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("vault missing nonce"))?;
    let cipher_b64 = vault["ciphertext"].as_str()
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>("vault missing ciphertext"))?;

    let salt = salt_b64.to_string();
    let nonce_bytes = general_purpose::STANDARD
        .decode(nonce_b64)
        .map_err(|_| PyErr::new::<pyo3::exceptions::PyValueError, _>("bad nonce"))?;
    let cipher_bytes = general_purpose::STANDARD
        .decode(cipher_b64)
        .map_err(|_| PyErr::new::<pyo3::exceptions::PyValueError, _>("bad ciphertext"))?;

    let cipher_nonce = XNonce::from_slice(&nonce_bytes);
    let trial_key = get_key(mpass.to_string(), salt);

    let cipher = XChaCha20Poly1305::new(Key::from_slice(&trial_key));
    match cipher.decrypt(cipher_nonce, cipher_bytes.as_ref()) {
        Ok(_) => {
            unsafe { ATTEMPTS = 0 };
            Ok("ok".into())
        }
        Err(_) => {
            unsafe {
                ATTEMPTS += 1;
                if ATTEMPTS >= 5 {
                    ATTEMPTS = 0;
                    Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                        "Too many failed attempts. Vault locked."))
                } else {
                    Ok(format!("Wrong password, {}/5 tries remaining", ATTEMPTS))
                }
            }
        }
    }
}

#[pymodule]
fn vault_core(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(pg1_startup, m)?)?;
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

fn get_key(arg_mpass: String, arg_salt: String) -> [u8; 32] {
    let mut key = [0u8; 32];
    let params = Params::new(65536, 3, 1, None).expect("Invalid Argon2 parameters");
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);
    argon2.hash_password_into(arg_mpass.as_bytes(), arg_salt.as_bytes(), &mut key).expect("Failed to derive key");
    return key;
}