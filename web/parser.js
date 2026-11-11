import init, { login_check, create_new_vault, fetch_db_info, get_unwrapped_key, decrypt } from "./pkg/ironvault.js";
const wasmReady = init();

const go_button = document.getElementById("go");
const cr_acc_btn = document.getElementById("cr_acc");
const user_box = document.getElementById("username");
const pass_box = document.getElementById("password");
const warning = document.getElementById("login-msg");
const pass_hint_box = document.getElementById("password_hint");
const s1 = document.querySelectorAll('.s1');
const s2 = document.querySelectorAll('.s2');
let username = "";
let warningTimer;
let vault_id = 0;
let vault_key = null;
let vault_plaintext = "";

function showWarning(message) {
    clearTimeout(warningTimer);
    warning.textContent = message;

    warningTimer = setTimeout(() => {
        warning.textContent = "";
    }, 5_000);
}

//user_info is [id, username, ciphertext, kdf_salt, nonce, password_hint]
function get_plaintext(user_info, pass) {
    const id = Number(user_info[0]);
    const ciphertext = user_info[2];
    const kdf_salt = user_info[3];
    const nonce = user_info[4];

    //As all accounts get encrypted data on creation, an empty ciphertext can't verify a password
    if (ciphertext === "") {
        showWarning("This account has no vault data to check a password against");
        return
    }

    //As get_unwrapped_key returns a key of all zeros on fail
    const key = get_unwrapped_key(pass, kdf_salt);
    if (key.every(byte => byte === 0)) {
        showWarning("Could not generate key from this account's salt");
        return
    }

    //As decrypt throws instead of returning a message if the nonce is the wrong length
    let plaintext = "";
    try {
        plaintext = decrypt(nonce, key, ciphertext);
    } catch {
        showWarning("This account's vault data is damaged");
        return
    }

    if (plaintext === "decryption error") {
        showWarning("Incorrect password");
        return
    }

    //All of the other messages decrypt can return in place of the plaintext
    if (plaintext === "invalid key length" || plaintext === "nonce parsing error" ||
        plaintext === "ciphertext parsing error" || plaintext === "plaintext parsing error") {
        showWarning(plaintext);
        return
    }

    //Only kept after a successful decrypt, so a wrong password's key is never stored
    vault_id = id;
    vault_key = key;
    vault_plaintext = plaintext;
    showWarning("Login successful");
}

cr_acc_btn.addEventListener('click', async () => {
    await wasmReady;
    username = user_box.value;
    const pass = pass_box.value;

    if (username === "" || pass === "") {
        showWarning("Missing Username or Password");
        return
    }

    const user_exits = await login_check(username);
    if (user_exits < 0) {
        showWarning("Could not reach the server, please try again");
    } else if (user_exits > 0) {
        showWarning("User already exists");
    } else {
        s1.forEach(el => {el.hidden = true});
        s2.forEach(el => {el.hidden = false});
        go_button.textContent = "Create Account"
    }
});

go_button.addEventListener('click', async () => {
    await wasmReady;
    if (go_button.textContent === "Login") {
        username = user_box.value;
        const pass = pass_box.value;

        if (username === "" || pass === "") {
            showWarning("Missing Username or Password");
            return
        }

        //As login check returns db id on success, 0 if the user doesn't exist, or -1 if the lookup failed
        const user_exits = await login_check(username);
        if (user_exits < 0) {
            showWarning("Could not reach the server, please try again");
        } else if (user_exits > 0) {
            let user_info = await fetch_db_info(user_exits);
            if (user_info.length > 1) {
                get_plaintext(user_info, pass);
            } else {
                showWarning(user_info[0]);
            }
        } else {
            showWarning("User does not exist")
        }
    } else {
        const pass_hint = pass_hint_box.value;
        if (pass_hint === "") {
            showWarning("Please enter a password hint for your account");
        } else {
            const err = await create_new_vault(username, pass_box.value, pass_hint);
            if (err != "") {
                showWarning(err);
            } else {
                s1.forEach(el => {el.hidden = false});
                s2.forEach(el => {el.hidden = true});
                go_button.textContent = "Login";
                pass_hint_box.value = "";
                pass_box.value = "";
                showWarning("Account created, you can now log in");
            }
        }
    }
});

[user_box, pass_box].forEach(input => {
    input.addEventListener('input', () => {
    });
});

//Should return the saved logins as [{ id, title, username, password }], called right after login
export async function load_entries() {}

//entry is the login being saved, next is the whole list of logins after the change
export async function save_entry(entry, is_new, next) {}

//ids are the selected logins, next is the whole list of logins without them
export async function delete_entries(ids, next) {}

export async function change_hint(hint) {}

export async function change_password(current, next) {}

//Called when the vault is locked, should forget vault_id, vault_key and vault_plaintext
export function lock_vault() {}
