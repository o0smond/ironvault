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

/*
By: Oliver Osmond
Date: 2025-11-23
Program Details: Password Manager
*/
fn main() {
    #![allow(warnings)]
    let mut mpass = String::new();
    let mut salt = String::new();
    let mut key = [0u8; 32];
    let storage_content = fs::read_to_string("src/storage.json").unwrap_or_else(|_| "{}".to_string());
    let mut vault: serde_json::Value = serde_json::from_str(&storage_content).unwrap_or(serde_json::json!({}));

    #[derive(Serialize, Deserialize, Debug)]
    struct Data {
        kdf_salt: Option<String>,
    }

    if let Some(existing_data) = vault.get("ciphertext").and_then(|v| v.as_str()) {
        println!("Salt found!");
        println!("Welcome back to Password Manager!");

        salt = vault
            .get("kdf_salt")
            .and_then(|v| v.as_str())
            .unwrap()
            .to_string();

        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 5;
        loop {
            attempts += 1;
            print!("Please enter your master password: ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("Failed to read line");
            let input = input.trim();
            if input.is_empty() {
                println!("Password cannot be blank. Please try again.\n");
                continue;
            }

            let trial_key = get_key(input.to_string(), salt.clone());

            let nonce_bytes = base64::decode(
                vault.get("nonce").and_then(|v| v.as_str()).unwrap()
            ).expect("bad nonce in file");
            let cipher_nonce = XNonce::from_slice(&nonce_bytes);

            let cipher = XChaCha20Poly1305::new(Key::from_slice(&trial_key));
            let ciphertext_bytes = base64::decode(existing_data)
                .expect("bad ciphertext in file");

            match cipher.decrypt(cipher_nonce, ciphertext_bytes.as_ref()) {
                Ok(plaintext_bytes) => {
                    mpass = input.to_string();
                    key   = trial_key;
                    let plaintext = String::from_utf8(plaintext_bytes)
                        .expect("decrypted data not utf-8");
                    println!("Decrypted passwords JSON");
                    break;
                }
                Err(_) => {
                    println!("Incorrect password, please try again. (attempt {}/{})", attempts, MAX_ATTEMPTS);
                    if attempts >= MAX_ATTEMPTS {
                        eprintln!("Too many failed attempts. Exiting.");
                        std::process::exit(1);
                    }
                }
            }
        }
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
        vault["kdf_salt"] = serde_json::Value::String(salt.clone());
        let json_string =
            serde_json::to_string_pretty(&vault).expect("Failed to serialize JSON");
        fs::write("src/storage.json", json_string).expect("Failed to write to storage.json");
        println!("Salt saved to JSON.");
    }
    println!("Generating key...");
    key = get_key(mpass, salt);
    println!("Key generated.");

    let nonce: Vec<u8> = if let Some(existing_nonce_b64) = vault.get("nonce").and_then(|v| v.as_str()) {
        base64::decode(existing_nonce_b64).expect("Failed to decode stored nonce")
    } else {
        println!("No nonce found. Generating new nonce...");
        let new_nonce = generate_nonce();
        vault["nonce"] = serde_json::Value::String(base64::encode(&new_nonce));
        new_nonce        
    };
    let cipher_nonce = XNonce::from_slice(&nonce);
    let cipher = XChaCha20Poly1305::new(Key::from_slice(&key));
    let plaintext: String;

    if let Some(existing_password_data) = vault.get("ciphertext").and_then(|v| v.as_str()) {
        let ciphertext_b64 = existing_password_data;

        let ciphertext_bytes = base64::decode(ciphertext_b64).expect("Failed to decode base64 ciphertext");

        let plaintext_bytes = cipher.decrypt(cipher_nonce, ciphertext_bytes.as_ref()).expect("Failed to decrypt ciphertext");

        plaintext = String::from_utf8(plaintext_bytes).expect("Decrypted plaintext invalid UTF-8");

        println!("Decryption successful");
    } else {
        println!("No previous passwords found.");
        plaintext = String::new();
    }

    let fresh_nonce = generate_nonce();
    vault["nonce"] = serde_json::Value::String(base64::encode(&fresh_nonce));

    let json_string = serde_json::to_string_pretty(&vault).expect("Failed to serialize JSON");
    fs::write("src/storage.json", json_string).expect("Failed to write updated nonce");
    
    let mut password_map: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&plaintext).unwrap_or_else(|_| serde_json::Map::new());
    let mut choice = choice_selector();
    run_editor(choice, &mut password_map);
    let new_nonce = generate_nonce();
    let new_nonce_x = XNonce::from_slice(&new_nonce);

    vault["nonce"] = serde_json::Value::String(base64::encode(&new_nonce));
    let new_ciphertext = cipher.encrypt(new_nonce_x, serde_json::to_string(&password_map).unwrap().as_bytes()).expect("Encryption failed");
    let new_ciphertext_b64 = base64::encode(&new_ciphertext);
    vault["ciphertext"] = serde_json::Value::String(new_ciphertext_b64);
    let json_string =
    serde_json::to_string_pretty(&vault).expect("Failed to serialize JSON");
    fs::write("src/storage.json", json_string).expect("Failed to write updated nonce");
    println!("Encryption successful, Closed successfully.");
}  

fn run_editor(choice: String, password_map: &mut serde_json::Map<String, serde_json::Value>) {
    if choice == "1" {
        println!("\n{}", serde_json::to_string_pretty(&password_map).unwrap());
        println!("\nWhat would you like to do next?");
        let choice_2 = choice_selector();
        run_editor(choice_2, password_map);
    } else if choice == "2" {
        println!("\nPlease enter the username of the password you would like to add:");
        let user = loop {
            print!("Username: ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).expect("Failed to read line");
            let input = input.trim();
            if input.is_empty() {
                println!("Username cannot be blank. Please try again.\n");
            } else if password_map.contains_key(input) == true {
                println!("Username already exists. Please try again.\n");
            } else {
                break input.to_string();
            }
        };
        println!("\nPlease enter the password you would like to add:");
        let pass = loop {
            print!("Password: ");
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

        password_map.insert(user, serde_json::Value::String(pass));
        let choice_2 = choice_selector();
        run_editor(choice_2, password_map);
    } else if choice == "3" {
        if serde_json::to_string_pretty(&password_map).unwrap() == "{}" {
          println!("\nNo passwords to edit/remove.");  
        } else {
            println!("\nPlease enter the username of the password you would like to remove/edit:");
            let user = loop {
                print!("Username: ");
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input).expect("Failed to read line");
                let input = input.trim();
                if input.is_empty() {
                    println!("Username cannot be blank. Please try again.\n");
                } else if input == "exit" || input == "Exit" || input == "EXIT" {
                    let choice_2 = choice_selector();
                    run_editor(choice_2, password_map);
                } else if password_map.contains_key(input) == false {
                    println!("Such a username does not exist. Please try again. To exit type 'exit'\n");
                } else {
                    break input.to_string();
                }
            };
            let choice_er = loop {
                print!("Edit or Remove? (E/R):");
                io::stdout().flush().unwrap();
                let mut input = String::new();
                io::stdin().read_line(&mut input).expect("Failed to read line");
                let input = input.trim();
                if input.is_empty() || input != "E" && input != "e" && input != "R" && input != "r" {
                    println!("Please enter E or R. Please try again.\n");
                } else {
                    break input.to_string();
                }
            };

            if choice_er == "E" || choice_er == "e" {
                let new_user = loop {
                    print!("New Username: ");
                    io::stdout().flush().unwrap();
                    let mut input = String::new();
                    io::stdin().read_line(&mut input).expect("Failed to read line");
                    let input = input.trim();
                    if input.is_empty() {
                        println!("Username cannot be blank. Please try again.\n");
                    } else {
                        break input.to_string();
                    }
                };
                let new_pass = loop {
                    print!("New Password: ");
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
                password_map.remove(&user);
                password_map.insert(new_user, serde_json::Value::String(new_pass));
                println!("Changes made.");
            } else if choice_er == "R" || choice_er == "r" {
                password_map.remove(&user);
                println!("Changes made.");
            }
        }
        let choice_2 = choice_selector();
        run_editor(choice_2, password_map);
    } else if choice == "4" {
        println!("Closing Password Manager...");
    }
}

fn generate_nonce() -> Vec<u8> {
    let mut nonce = [0u8; 24];
    rand::thread_rng().fill_bytes(&mut nonce);
    nonce.to_vec()
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