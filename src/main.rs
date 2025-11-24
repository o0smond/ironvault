use std::{fs, io::{self, Write}};
use serde::{Serialize, Deserialize};
use serde_json;
use rand::Rng;
use rand::RngCore;
use base64::{engine::general_purpose, Engine as _};
use argon2::{Argon2, password_hash::{SaltString}, Params, PasswordHasher};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use chacha20poly1305::aead::{Aead, NewAead};
use rand_core::OsRng;

/*
By: Oliver Osmond
Date: 2025-11-23
Program Details: Password Manager
*/
fn main() {
    let storage_content = fs::read_to_string("src/storage.json").unwrap_or_else(|_| "{}".to_string());
    let mut mpass = String::new();
    let mut salt = String::new();
    let mut key = [0u8; 32];

    #[derive(Serialize, Deserialize, Debug)]
    struct Data {
        salt: Option<String>,
        passwords: serde_json::Value
    }

    let data: Data = serde_json::from_str(&storage_content).unwrap_or(Data {
        salt: None,
        passwords: serde_json::Value::Null
    }); 



    

    if let Some(existing_salt) = data.salt{
        println!("Salt found!");
        println!("Welcome back to Password Manager!");
        salt = existing_salt;
    } else {
        println!("\nNo salt found, starting fresh run.");
        println!("\nHello, welcome to Password Manager!\nTo begin, please create a master password.\nNote, this will be used to access all future passwords so if you forget it, your passwords will be inaccessible.\n");
        mpass = loop {
            print!("Please enter your master password: ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("Failed to read line");
            let input = input.trim();
            if input.is_empty() {
                println!("Password cannot be blank. Please try again.\n");
            } else {
                break input.to_string();
            }
        };
    
        println!("Your master password is: {}", mpass);

        println!("Your salt is being generated...");
        salt = generate_salt(16);
        println!("Your salt is: {}", salt);

        let mut vault: serde_json::Value = serde_json::from_str(&storage_content).unwrap_or(serde_json::json!({}));
        vault["kdf_salt"] = serde_json::Value::String(salt.clone());

        let json_string = serde_json::to_string_pretty(&vault).expect("Failed to serialize JSON");
        fs::write("src/storage.json", json_string).expect("Failed to write to storage.json");
        println!("Salt saved to JSON.");
    }
    println!("Generating key...");
    key = get_key(mpass, salt);
    println!("Your key is: {}", general_purpose::STANDARD.encode(&key));

    let mut plaintext = String::new();

    if let Some(existing_password_data) = data.passwords.as_object() {
        println!("Passwords found!");
        let nonce = generate_nonce();
        let cipher_nonce = Nonce::from_slice(&nonce);
        let cipher_key = Key::from_slice(&key);
        let cipher = ChaCha20Poly1305::new(cipher_key);
        let ciphertext = existing_password_data["ciphertext"].as_str().unwrap();
        let ciphertext_b64 = existing_password_data["ciphertext"].as_str().expect("ciphertext missing");
        // Decode base64 → Vec<u8>
        let ciphertext_bytes = base64::decode(ciphertext_b64)
        .expect("Failed to decode base64 ciphertext");

        // Decrypt
        let plaintext_bytes = cipher
        .decrypt(cipher_nonce, ciphertext_bytes.as_ref())
        .expect("Decryption failed");

        // Convert bytes → string
        plaintext = String::from_utf8(plaintext_bytes)
        .expect("Decrypted plaintext is not valid UTF-8");

        println!("Decrypted passwords JSON:\n{}", plaintext);

    } else {
        println!("No previous passwords found.");
        plaintext = serde_json::json!({"test": "test"}).to_string();
    }
    
    let password_map: serde_json::Value = serde_json::from_str(&plaintext).expect("Invalid JSON");
    let mut choice = choice_selector();
    let mut to_be_added = String::new();

    if (choice == "1") {
        println!("\n{}", password_map);
    } else if (choice == "2") {
        {}
    } else if (choice == "3") {
        {}
    } else if (choice == "4") {
        println!("Goodbye!");
        std::process::exit(0);
    }
}  

fn generate_nonce() -> [u8; 24] {
    let mut nonce = [0u8; 24];
    rand::thread_rng().fill_bytes(&mut nonce);
    return nonce;
}

fn choice_selector() -> String {
    println!("\n Please choose what you would like to do:");
    println!("1. View current passwords");
    println!("2. Add a new password");
    println!("3. Remove/Edit a password");
    println!("4. Exit");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    return input.trim().to_string();
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