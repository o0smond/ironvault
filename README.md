# IronVault

A Rust based password manager in the browser. All encryption is handled clientside with WebAssembly, so only ciphertext is ever stored.

[![Live Demo](https://img.shields.io/badge/demo-live-brightgreen)](https://ironvault.oosmond.site)
![Rust](https://img.shields.io/badge/Rust-WASM-orange?logo=rust)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

---

<img width="1200" height="705" alt="demo" src="https://github.com/user-attachments/assets/e9c3cf31-ab74-42e7-bfa6-00c944a88bc4" />

## Overview

Password managers are highly trust-heavy, relying on one password to safeguard all others stored. Ironvault aims to be a simple, 
lightweight alternative, that passes all sensitive processing directly to the client.
Ironvault is a password manager built in Rust, vanilla JS and tailwind CSS. It relies on a JavaScript cloudflare worker,
and a Turso database to quickly handle user requests and access to the database. Because everything happens clientside, the only things ever moving 
in and out of your device are Argon2/XChaCha20-Poly1305 encrypted strings of ciphertext; all encryption and decryption is handled clientside through
WebAssemby compiled Rust. This branch is the most current and polished version of Ironvault, with past desktop versions being written in purely
Rust (CLI and Macroquad Editions) and python (using pyQT with maturin), and storing encrypted information locally.

## Features

- Create a vault that locks with a master password
- Add, edit, or delete entries
- Change master password or hint
- Password hint after two failed logins
- No third-party JS loaded at runtime (Tailwind is compiled locally; not from a CDN)


## Tech Stack

| Layer | Technology |
|-------|------------|
| Crypto core | Rust → WebAssembly (`wasm-bindgen`, `wasm-pack`) |
| KDF | Argon2id |
| Encryption | XChaCha20-Poly1305 (AEAD) |
| Frontend | Vanilla JS (ES modules), Tailwind CSS (compiled with the CLI) |
| API | Cloudflare Workers |
| Database | Turso (libSQL); credentials stored as Cloudflare Worker secrets |

## Architecture
<img width="1500" height="775" alt="ivault_login" src="https://github.com/user-attachments/assets/5c5dd99e-2276-4d7b-9d32-7894df18c756" />
Example data flow upon locking of the vault

### ui.js:
Contains all frontent JavaStript and calls the appropriate JS function from
parser.js based on the frontend input. Passes scraped data from inputs
(username, password, etc.) to parser.js.

### parser.js:
Responsible for orchestrating the calling of all the WASM-compiled Rust functions.
Conisists of mainly functions that get called by ui.js on frontend events.
Operates off the principle that the Rust core is simply responsable for processing,
and passes the current state of the vault as a stringified list to Rust when needed,
along with whatever else is needed. 

### lib.rs (and general Rust core):
Responsable for compute-heavy tasks (key derivation, encyption and decryption, etc.)
and any contact with the cloudflare worker, through helper file database.rs. Consists
of functions that get compiled to WASM, and called though their bindings by parser.js.
Parser passes any nessisary data for the function, with the current decrypted vault
being passed as a stringified list.

### Cloudflare Worker:
A JavaScript worker that receives an HTTPS POST from the clientside Rust, and a JSON
packet containing an action and things it will need for it. This then gets translated
to SQL, and run on the TursoDB though the URL and key stored as enviroment secrets.
After running the SQL, whatever is needed is returned to the client through JSON packet.

### Turso:
A libSQL database that is accessed through SQL from the Cloudflare worker. It is responsable
for storing data on each user, with each row containing the user's ID, username, their encrypted
vault as base64 ciphertext, their salt, nonce, and finally their password hint. Note that username 
and password hint are stored unencrypted.

## Security Model

<!--
Keep the "why" style from the old README, but write it about THIS design. Subsections:
-->

### What the server sees
<!-- Username, ciphertext, KDF salt, nonce, password hint. Never the master password or the key. -->

### Key derivation: Argon2id
<!-- Why Argon2id, plus your actual parameters (lib.rs: 64 MiB memory, 3 iterations, 1 lane) and why you chose them. -->

### Encryption: XChaCha20-Poly1305
<!--
Why ChaCha over AES in a browser (constant-time in software, no AES-NI from WASM).
Why the X variant (192-bit random nonces are safe to generate at random). What the Poly1305 tag protects against.
-->

### Threat model & known limitations
<!--
Most student projects skip this. Writing it honestly makes you look senior. Be upfront about:
- The worker has open CORS and no per-user auth, so anyone can request any user's ciphertext.
  It's still encrypted, so it becomes an offline brute-force target protected only by Argon2.
- Password hints are stored in plaintext.
- Trusting the server to deliver unmodified JS/WASM (true of every web-based password manager).
- What you would do next to fix each one (see Roadmap).
-->

## Getting Started

### Prerequisites
<!-- Rust stable (version), the wasm32-unknown-unknown target, wasm-pack, Node (only for Tailwind), a static file server. -->

### Build & run
```bash
git clone https://github.com/o0smond/ironvault.git
cd ironvault
npm install
npm run build:css
helpers/build_wasm.sh
```
<!-- Then: how to serve /web locally (e.g. `python -m http.server -d web`), and which URL to open. -->

### Deploying the worker (optional)
<!-- wrangler steps, and the env vars it needs: DB_URL and DB_TOKEN (your Turso database URL and auth token), set with `wrangler secret put`. Don't put the values here. -->

## Project Structure

```
rust_core/   Rust crypto + DB client, compiled to WASM
web/         Static frontend (index.html, ui.js, parser.js, Tailwind)
worker/      Cloudflare Worker: HTTP bridge to the database
helpers/     Build scripts
```
<!-- One line per folder is enough. This replaces the old "Codebase Tour" screenshots. -->

## Challenges & What I Learned

<!--
Write this as 2–4 short engineering stories, not a list of skills. Each one: problem → what you tried → what you chose and why.
Good candidates from your commit history:
- Deciding where the decrypted vault lives (moved from ui.js to parser.js) and how JS and Rust exchange data (JSON strings).
- Removing the Tailwind CDN because it was a supply-chain risk for a password manager.
- Master password change: re-deriving the key and re-encrypting safely.
- Porting from native (ureq) to WASM (gloo-net) with cfg-gated targets.
Interviewers often ask about exactly these stories, so write ones you can talk about for 5 minutes.
-->

## Roadmap

<!-- 3–5 checkboxes. Shows you know what's missing. e.g. per-user auth on the worker, rate limiting, CSP headers, tests for the crypto round-trip, password generator. -->

- [ ] 
- [ ] 

## Previous Edition

<!-- One line + link to the Macroquad desktop version (branch or tag). This keeps that work without cluttering this README. -->

## License

<!-- MIT is the usual choice. Add a LICENSE file too. -->
