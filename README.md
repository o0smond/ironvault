<div align="center">
<pre>
██╗██████╗  ██████╗ ███╗   ██╗██╗   ██╗ █████╗ ██╗   ██╗██╗     ████████╗
██║██╔══██╗██╔═══██╗████╗  ██║██║   ██║██╔══██╗██║   ██║██║     ╚══██╔══╝
██║██████╔╝██║   ██║██╔██╗ ██║██║   ██║███████║██║   ██║██║        ██║&nbsp;&nbsp;&nbsp;
██║██╔══██╗██║   ██║██║╚██╗██║╚██╗ ██╔╝██╔══██║██║   ██║██║        ██║&nbsp;&nbsp;&nbsp;
██║██║  ██║╚██████╔╝██║ ╚████║ ╚████╔╝ ██║  ██║╚██████╔╝███████╗   ██║&nbsp;&nbsp;&nbsp;
╚═╝╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═══╝  ╚═══╝  ╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝&nbsp;&nbsp;&nbsp;
</pre>
</div>

<p align="center">A Rust-based password manager in the browser. All encryption is handled client-side with WebAssembly, so only ciphertext is ever stored.</p>

<p align="center">
  <a href="https://ironvault.oosmond.site"><img src="https://img.shields.io/badge/demo-live-brightgreen" alt="Live Demo"></a>
  <img src="https://img.shields.io/badge/Rust-WASM-orange?logo=rust" alt="Rust">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue" alt="License: MIT"></a>
</p>

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

## Security Architecture
### What the server sees
Username, KDF salt, nonce, password hint, and encrypted ciphertext. Never a password or the key.

### Key derivation: Argon2id
Used in the Rust core (lib.rs) for key derivation from the inputted password and stored salt from the database.
Argon2id was chosen as OWASP lists it as their top choice for a KDF (https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html),
and it integrates easily with Rust. Ironvault configures Argon to use 64MiB of memory and 3 passes every guess, which
makes it extremely hard for GPUs to crack, and sits well above OWASP's recomended settings.

### Encryption: XChaCha20-Poly1305
The vault is encrypted with XChaCha20 and authenticated with a Poly1305 tag, so any tampering with the stored ciphertext is detected and 
decryption fails. I chose it over AES because WebAssembly has no access to hardware AES instructions, and ChaCha20 is faster for this case 
(in a browser sandbox). The extended 24-byte nonce makes random nonces safe.

### Threat model & known limitations
<!--
Most student projects skip this. Writing it honestly makes you look senior. Be upfront about:
- The worker has open CORS and no per-user auth, so anyone can request any user's ciphertext.
  It's still encrypted, so it becomes an offline brute-force target protected only by Argon2.
- Password hints are stored in plaintext.
- Trusting the server to deliver unmodified JS/WASM (true of every web-based password manager).
- What you would do next to fix each one (see Roadmap).
-->
- The worker has open CORS and no per-user auth, so anyone can use the database API if they can
  figure out the calls.
- Password hints are plaintext
- Trusting that the server will deliver unmodified JS/WASM
- See Roadmap for fixes/next steps

## Getting Started
On my website at https://ironvault.oosmond.site. Or, if you'd prefer to host locally with your own database

### Prerequisites
Rust stable (version), the wasm32-unknown-unknown target, wasm-pack, Node (only for Tailwind), a static file server.

### Build & run
```bash
git clone https://github.com/o0smond/ironvault.git
cd ironvault
npm install
npm run build:css
helpers/build_wasm.sh
```
Then, serve locally or host on cloudflare or however you prefer,
```bash
python -m http.server -d web
```

### Deploying the worker & Turso Setup (Only for personal setup)
<!-- wrangler steps, and the env vars it needs: DB_URL and DB_TOKEN (your Turso database URL and auth token), set with `wrangler secret put`. Don't put the values here. -->
Head to https://app.turso.tech/login or your prefered database serivce. Create a new table named 'ironvault', and rows as 'id',
'username', 'ciphertext', 'kdf_salt', 'nonce', and 'password_hint'.

<img width="1205" height="39" alt="image" src="https://github.com/user-attachments/assets/912a3b31-87d1-456a-b235-967395d03c4a" />

Next, go to cloudflare.com and click 'Dashboard'. After logging in, create a new worker, and paste the worker JS code
under /worker/worker.js. Then, navigate to settings and add two ENV variables, 'DB_TOKEN' and 'DB_URL'. The vaules for
these are the provided by Turso when you generate a new key; ensure to switch the libsql:// to https:// in the URL Turso gives.

Finally, navigate back to /rust_core/lib.rs and replace the current URL with your new Turso one. Re-compile the Rust core with the
build_wasm.sh helper script, and if all goes well, you should have IronVault running on your own custom database!

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
