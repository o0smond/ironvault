const go_button = document.getElementById("go");
const user_box = document.getElementById("username");
const pass_box = document.getElementById("password");
const warning = document.getElementById("login-msg");
let warning_event = false;

go_button.addEventListener('click', () => {
    const user = user_box.value;
    const pass = pass_box.value;

    if (user === "" || pass === "") {
        warning.textContent = "Missing Username or Password";
        warning_event = true;
    }
});

[user_box, pass_box].forEach(input => {
    input.addEventListener('input', () => {
        if (warning_event) {
            warning.textContent = "";
            warning_event = false;
        }
    });
});