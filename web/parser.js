import init, { login_check, create_new_vault, fetch_db_info, get_unwrapped_key, decrypt, encrypt_and_push } from "./pkg/ironvault.js";
const wasmReady = init();

const go_button = document.getElementById("go");
const cr_acc_btn = document.getElementById("cr_acc");
const user_box = document.getElementById("username");
const pass_box = document.getElementById("password");
const warning = document.getElementById("login-msg");
const pass_hint_box = document.getElementById("password_hint");
const s1 = document.querySelectorAll('.s1');
const s2 = document.querySelectorAll('.s2');
let warningTimer;
let vault_id = 0;
let username = "";
let password_hint = "";
let vault_key = null;
let entries = []; //decrypted vault as [{ id, title, username, password }]

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

    //As new accounts are created with an encrypted empty string, "" means no logins yet
    let parsed = [];
    try {
        parsed = plaintext ? JSON.parse(plaintext) : [];
    } catch {
        showWarning("This account's vault data is damaged");
        return
    }

    //Only kept after a successful decrypt, so a wrong password's key is never stored
    vault_id = id;
    vault_key = key;
    entries = parsed;
    showWarning("Login successful");
    document.dispatchEvent(new Event("unlocked"));
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
                password_hint = user_info[5];
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

let pushing = false;

//Pushes the whole vault and the hint. Only keeps them once the DB write worked, so a failed save never shows as saved
async function push_vault(next, hint = password_hint) {
    //As two pushes at once would both start from the same old list, and one would be lost
    if (pushing) return "Still saving, try again";
    pushing = true;
    try {
        const r = await encrypt_and_push(vault_id, JSON.stringify(next), vault_key, hint);
        if (r !== "OK") return r;
        //As locking during the save already wiped the vault, so don't put the entries back
        if (!vault_key) return "Vault was locked";
        entries = next;
        password_hint = hint;
    } finally {
        pushing = false;
    }
}

export function get_entries() {
    return entries;
}

//Adds the entry if its id is new, otherwise replaces the old one with the same id
export async function save_entry(entry) {
    const new_entries = [...entries];
    const index = new_entries.findIndex(existing_entry => existing_entry.id === entry.id);
    if (index === -1) {
        new_entries.push(entry);
    } else {
        new_entries[index] = entry;
    }
    return push_vault(new_entries);
}

export async function delete_entries(ids) {
    return push_vault(entries.filter(e => !ids.includes(e.id)));
}

//Pushed now rather than at lock, as closing the tab never runs lock_vault
export async function change_hint(hint) {
    return push_vault(entries, hint);
}

//As returning nothing would show "Master password changed" in the UI
export async function change_password(current, next) {
    return "Changing the master password isn't built yet";
}

export function lock_vault() {
    vault_id = 0;
    vault_key = null;
    entries = [];
    password_hint = "";
}
