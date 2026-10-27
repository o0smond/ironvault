import init, { login_check, create_new_vault } from "./pkg/ironvault.js";
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

function showWarning(message) {
    clearTimeout(warningTimer);
    warning.textContent = message;

    warningTimer = setTimeout(() => {
        warning.textContent = "";
    }, 5_000);
}

cr_acc_btn.addEventListener('click', async () => {
    await wasmReady;
    const user_exits = await login_check(username);
    if (user_exits) {
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

        const user_exits = await login_check(username);
        if (user_exits) {
            //Todo main fxn
        } else {
            showWarning("User does not exist")
        }
    } else {
        const pass_hint = pass_hint_box.value;
        if (pass_hint === "") {
            showWarning("Please enter a password hint for your account");
        } else {
            const err = await create_new_vault(username, pass_hint);
            if (err != "") {
                showWarning(err);
            }
        }
    }
});

[user_box, pass_box].forEach(input => {
    input.addEventListener('input', () => {
    });
});