# IronVault

A visually driven, security focused password manager built entirely in Rust.

This program utilizes the Macroquad engine to render a hardware-accelerated, lightweight
GUI that proves modern security software can work well and look good too. Built with a focus
on memory safety and modern cryprographic primatives, this project is free from the security
concerns of storing passwords on the web.

![ironvault](https://github.com/user-attachments/assets/b892576e-3e16-4fbb-8a3e-3e83e14b0403)

---

## Tech Stack:

-**Language:** Rust (Stable)

-**GUI Engine:** Macroquad

-**KDF:** Argon2id (Memory-hard key derivation)

-**Encryption:** ChaCha20-Poly1305 (Authenticated Encryption)

-**Serialization:** The vault is serialized into a JSON file as a b64 string. This allows for great security while ensuring ease of storage.

---

## Codebase Tour:

This codebase is broken into two large parts, consisting of:
- **/src/main.rs** (Frontend, GUI Handling, See main.rs comments)
<img width="270" height="217" alt="image" src="https://github.com/user-attachments/assets/7eb82a5a-b4e8-4efa-81fd-ddf6d3cf5089" />

- **/src/modules/vault_core/mod.rs** (Backend, self-contained vault manager to interact with the storage.JSON file in the same directory. For more details see mod.rs's comments at the specified directory)

<img width="270" height="376" alt="image" src="https://github.com/user-attachments/assets/b2cfaea0-5919-42ac-ac7a-f44a84dd5194" />

---

## Security Architecture

1. **The KDF: Argon2id**

   **Why:** It is the winner of the Password Hashing Competition and is very resistant to GPU cracking side-channel attacks, as well as brute force attacks.
2. **The Encryption: ChaCha20-Poly1305**
   
   **Why:** Unlike the global AES, ChaCha20 is more resistant to cache-timing attacks.
   Furthermore, the Poly1305 provides an authentication tag for the data, ensuring the data
   has not been tampered with since the last encryption.

---

## Running the Project

Since I used the Macroquad engine, this project is cross-platform.

**Dependencies:**

This program requires Rust [Stable 1.75.0 or Newer]

**Clone the github repository:**

git clone https://github.com/o0smond/ironvault.git

**Run the project:**

cargo run --release

---

## What I learned

As this project's main purpose was learning, I have learned many, many things both about Rust itself and modern cryptographic techniques.

1. **Experience with modern cryptography:** This project demanded knowledge of modern cryptographic techniques such as salt and nonce generation, key derivation, and proper encryption and handling of the vault data.
   
2. **Experience with Rust:** As is becoming more evident in recent years, **Rust is the future**, due to its memory safe properties, along with other benefits. Getting hands on experience with this language which sits at the forefront of modern technonlogy has given me invaluable experience and has set me up nicely for a future in tech.
   
3. **Experience with UI Design** Clean, modern GUIs are essential for any truly user-freindly peice of software. Getting experience with the Macroquad Engine has given me skills in an immediate mode GUI system in contrast to my experience with PyQT's retained mode GUI system.
