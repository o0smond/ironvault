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

## Please Note
This project was developped for learning purposes only, and has not been independantly audited. 
IronVault is not intended to store serious or sensitive data.


## Overview

Password managers are highly trust-heavy, relying on one password to safeguard all the others stored. IronVault aims to be a simple,
lightweight alternative that passes all sensitive processing directly to the client.
IronVault is a password manager built in Rust, vanilla JS and Tailwind CSS. It relies on a JavaScript Cloudflare Worker
and a Turso database to quickly handle user requests and access to the database. Because everything happens client-side, the only things ever moving
in and out of your device are Argon2/XChaCha20-Poly1305 encrypted strings of ciphertext; all encryption and decryption is handled client-side through
WebAssembly-compiled Rust. This branch is the most current and polished version of IronVault, with past desktop versions being written purely in
Rust (CLI and Macroquad Editions) and Python (using PyQt with maturin), and storing encrypted information locally.

## Features

- Create a vault that locks with a master password
- Add, edit, or delete entries
- Change master password or hint
- Password hint after two failed logins
- No third-party JS loaded at runtime (Tailwind is compiled locally, not loaded from a CDN)


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
Contains all frontend JavaScript and calls the appropriate JS function from
parser.js based on the frontend input. Passes scraped data from inputs
(username, password, etc.) to parser.js.

### parser.js:
Responsible for orchestrating the calling of all the WASM-compiled Rust functions.
Consists mainly of functions that get called by ui.js on frontend events.
Operates on the principle that the Rust core is simply responsible for processing,
and passes the current state of the vault as a stringified list to Rust when needed,
along with whatever else is needed.

### lib.rs (and general Rust core):
Responsible for compute-heavy tasks (key derivation, encryption and decryption, etc.)
and any contact with the Cloudflare Worker, through the helper file database.rs. Consists
of functions that get compiled to WASM, and called through their bindings by parser.js.
Parser passes any necessary data for the function, with the current decrypted vault
being passed as a stringified list.

### Cloudflare Worker:
A JavaScript worker that receives an HTTPS POST from the client-side Rust, and a JSON
packet containing an action and the data it needs. This then gets translated
to SQL, and run on the Turso DB through the URL and key stored as environment secrets.
After running the SQL, whatever is needed is returned to the client as a JSON packet.

### Turso:
A libSQL database that is accessed through SQL from the Cloudflare Worker. It is responsible
for storing data on each user, with each row containing the user's ID, username, their encrypted
vault as base64 ciphertext, their salt, nonce, and finally their password hint. Note that username
and password hint are stored unencrypted.

## Security Architecture
### What the server sees
Username, KDF salt, nonce, password hint, and encrypted ciphertext. Never a password or the key.

### Key derivation: Argon2id
Used in the Rust core (lib.rs) for key derivation from the inputted password and stored salt from the database.
Argon2id was chosen as OWASP lists it as their top choice for a KDF (https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html),
and it integrates easily with Rust. IronVault configures Argon2 to use 64 MiB of memory and 3 passes for every guess, which
makes brute-forcing expensive even on GPUs, and sits well above OWASP's recommended settings.

### Encryption: XChaCha20-Poly1305
The vault is encrypted with XChaCha20 and authenticated with a Poly1305 tag, so any tampering with the stored ciphertext is detected and
decryption fails. I chose it over AES because WebAssembly has no access to hardware AES instructions, and ChaCha20 is faster in that case
(in a browser sandbox). The extended 24-byte nonce makes random nonces safe.

### Threat model & known limitations
- The worker has open CORS and no per-user auth, so anyone can use the database API if they can
  figure out the calls.
- Password hints are plaintext
- Trusting that the server will deliver unmodified JS/WASM

## Getting Started
Use it on my website at https://ironvault.oosmond.site, or, if you'd prefer, host it locally with your own database.

### Prerequisites
Rust (stable), the wasm32-unknown-unknown target, wasm-pack, Node (only for Tailwind), a static file server.

### Build & run
```bash
git clone https://github.com/o0smond/ironvault.git
cd ironvault
npm install
npm run build:css
helpers/build_wasm.sh
```
Then serve `web/` locally (and open http://localhost:8000), or host it on Cloudflare or however you prefer:
```bash
python -m http.server -d web
```

### Deploying the worker & Turso setup (only for a personal setup)
Head to https://app.turso.tech/login or your preferred database service. Create a new table named 'ironvault', with the columns 'id',
'username', 'ciphertext', 'kdf_salt', 'nonce', and 'password_hint'.

<img width="1205" height="39" alt="image" src="https://github.com/user-attachments/assets/912a3b31-87d1-456a-b235-967395d03c4a" />

Next, go to cloudflare.com and click 'Dashboard'. After logging in, create a new worker, and paste the worker JS code
from /worker/worker.js. Then, navigate to settings and add two environment variables, 'DB_TOKEN' and 'DB_URL'. The values for
these are provided by Turso when you generate a new token; make sure to switch the libsql:// in the URL Turso gives to https://.

Finally, open /rust_core/src/utils/database.rs and replace `WORKER_URL` with your new worker's URL. Re-compile the Rust core with the
build_wasm.sh helper script, and if all goes well, you should have IronVault running on your own custom database!

## Project Structure

```
rust_core/   Rust crypto + DB client, compiled to WASM
web/         Static frontend (index.html, ui.js, parser.js, Tailwind)
worker/      Cloudflare Worker: HTTP bridge to the database
helpers/     Build scripts
```

## Challenges & What I Learned
### Deciding where vault info should live
In the previous version of IronVault, all information on the vault, such as the key, salt, and decrypted
plaintext, was stored with the Rust core and other Rust functions. When working on this redesign, I made
the decision to move all storage of these variables to a JavaScript file that orchestrates all the Rust functions
instead (parser.js). This decision was made largely because JS makes it much easier to work with variables
outside the scope of a function, and it makes a lot more sense to connect the raw Rust functions with the frontend using
a JS pseudo-glue bridge like parser. Overall, this project has helped me hone my full-stack web development skills, and
I have come to realize that for most tasks, this architecture (only using Rust for compute-heavy tasks) makes a lot more sense
for the modern web environment.

### Deciding to create this rewrite
The previous three editions of IronVault were all desktop applications that stored information in JSON vaults.
The decision to rewrite IronVault largely stems from the fact that, going into this rewrite, I had become much
more comfortable developing in Rust, using Rust for web applications, and developing full-stack web applications.
The previous Macroquad edition was fairly heavy and clunky, and while it did work, it was difficult to set up and was not convenient
to use whatsoever. My hope is that with a web app, IronVault can actually become a viable and convenient option for storing logins.


## Future plans/add-ons
- [ ] Per-user auth on the worker/greater API security
- [ ] Password generator; create strong passwords for given logins
- [ ] Hashing password hints
- [ ] Chrome extension; unlock the vault, and fill saved logins only on matching domains, reusing the Rust/WASM core

## Previous Edition
Macroquad Edition:
https://github.com/o0smond/ironvault/tree/Macroquad-Edition

## License
This project is licensed under the [MIT License](LICENSE).
