use std::{fs, io::{self, Write}};
use serde::{Serialize, Deserialize};
use serde_json;
use rand::Rng;
use base64::{engine::general_purpose, Engine as _};
use argon2::{Argon2, password_hash::{SaltString}, Params, PasswordHasher};
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
    println!("\nYour key is: {}", general_purpose::STANDARD.encode(&key));

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