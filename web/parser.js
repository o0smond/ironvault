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
const show_hint = document.getElementById("show_hint");
const hint_box = document.getElementById("hint_box");
let warningTimer;
let failed_attempts = 0;
let hint_user = "";
let username = "";
let vault_key = null;
let entries = []; //decrypted vault as [{ id, title, username, password }]
let vault_user_info; //user_info is [id, username, ciphertext, kdf_salt, nonce, password_hint]

function showWarning(message) {
    clearTimeout(warningTimer);
    warning.textContent = message;

    warningTimer = setTimeout(() => {
        warning.textContent = "";
    }, 5_000);
}

//user_info is [id, username, ciphertext, kdf_salt, nonce, password_hint]
//Returns an error string, or { key, entries } on success. Changes no state, so a failed check never touches the open vault
function verify_password(user_info, pass) {
    const ciphertext = user_info[2];
    const kdf_salt = user_info[3];
    const nonce = user_info[4];

    //As all accounts get encrypted data on creation, an empty ciphertext can't verify a password
    if (ciphertext === "") {
        return "This account has no vault data to check a password against";
    }

    //As get_unwrapped_key returns a key of all zeros on fail
    const key = get_unwrapped_key(pass, kdf_salt);
    if (key.every(byte => byte === 0)) {
        return "Could not generate key from this account's salt";
    }

    //As decrypt throws instead of returning a message if the nonce is the wrong length
    let plaintext = "";
    try {
        plaintext = decrypt(nonce, key, ciphertext);
    } catch {
        return "This account's vault data is damaged";
    }

    if (plaintext === "decryption error") {
        return "Incorrect password";
    }

    //All of the other messages decrypt can return in place of the plaintext
    if (plaintext === "invalid key length" || plaintext === "nonce parsing error" ||
        plaintext === "ciphertext parsing error" || plaintext === "plaintext parsing error") {
        return plaintext;
    }

    //As new accounts are created with an encrypted empty string, "" means no logins yet
    let parsed = [];
    try {
        parsed = plaintext ? JSON.parse(plaintext) : [];
    } catch {
        return "This account's vault data is damaged";
    }

    return { key, entries: parsed };
}

//Hides the hint and restarts the failed attempt count for the current username
function reset_hint() {
    show_hint.hidden = true;
    hint_box.hidden = true;
    failed_attempts = 0;
    hint_user = username;
}

show_hint.addEventListener('click', () => {
    show_hint.hidden = true;
    hint_box.hidden = false;
});

function get_plaintext(user_info, pass) {
    const result = verify_password(user_info, pass);
    if (typeof result === "string") {
        if (result === "Incorrect password" && ++failed_attempts >= 2 && hint_box.hidden) {
            //textContent, so the hint can't inject HTML
            hint_box.textContent = `Password hint: "${user_info[5]}"`;
            show_hint.hidden = false;
        }
        return showWarning(result);
    }
    reset_hint();
    //Only kept after a successful decrypt, so a wrong password's key is never stored
    vault_key = result.key;
    entries = result.entries;
    vault_user_info = user_info;
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
        reset_hint();
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
        //So one user's failed attempts never reveal another user's hint
        if (username !== hint_user) reset_hint();

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

let pushing = false;

//Pushes the whole vault and the hint. Only keeps them once the DB write worked, so a failed save never shows as saved
async function push_vault(next, hint = vault_user_info[5], key = vault_key) {
    //As two pushes at once would both start from the same old list, and one would be lost
    if (pushing) return "Still saving, try again";
    pushing = true;
    try {
        const r = await encrypt_and_push(Number(vault_user_info[0]), JSON.stringify(next), key, hint);
        if (r !== "OK") return r;
        //As locking during the save already wiped the vault, so don't put the entries back
        if (!vault_key) return "Vault was locked";
        entries = next;
        vault_user_info[5] = hint;
        vault_key = key;
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

//Returns "" on success or an error string. Salt is reused on purpose: it's already unique per account, so only ciphertext and nonce change
export async function change_password(current, next) {
    //Gening a new key with the same salt with their old pass and comparing byte by byte.
    const check = get_unwrapped_key(current, vault_user_info[3]);
    if (!check.every((byte, i) => byte === vault_key[i])) return "Incorrect current password";

    //Updating key
    const new_key = get_unwrapped_key(next, vault_user_info[3]);
    if (new_key.every(byte => byte === 0)) return "Could not generate key from this account's salt";
    
    //push_vault updates global key w new_key here on success so no need to here
    const err = await push_vault(entries, undefined, new_key);
    if (err === undefined || err === "Vault was locked") return "";
    return err;
}

export function lock_vault() {
    vault_key = null;
    entries = [];
    vault_user_info = null;
}
