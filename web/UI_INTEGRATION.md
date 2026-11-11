# IronVault web UI — how it works and how to wire it up

| File | What it is |
|---|---|
| `index.html` | All four views as markup, the Tailwind theme, and the Heroicons sprite |
| `ui.js` | View switching, list rendering, selection, and the entry/account forms. Calls parser.js for every change |
| `parser.js` | Your login code (unchanged), plus six blank vault-action functions at the bottom for you to fill in |

`pkg/` and the Rust code are unchanged. The old `style.css` was removed because Tailwind replaces it.

Run it with any static server from `web/`, for example `python3 -m http.server 8123` (wasm will not load from `file://`).

## Views

The page has a single `<section data-view="…">` per screen. `showView(name)` in `ui.js` shows one and hides the rest using the `hidden` attribute.

| View | From the sketch | Contents |
|---|---|---|
| `login` | Panel 1 | The original login and create-account form, plus the footer |
| `list` | Panel 2 | Toolbar (menu, account, edit selected, delete selected, add), then the saved logins with checkboxes |
| `entry` | Panel 3 | Back arrow, then title, username and password. Each has a pencil to edit, and the password has an eye to show it. Also used for adding a new login |
| `account` | (new) | Change password hint, change master password, lock vault |

## Compatibility with parser.js's login code

The login code in parser.js only touches the DOM through these hooks, and all of them are kept exactly:

- ids `go`, `cr_acc`, `username`, `password`, `password_hint`, `login-msg`
- `.s1` / `.s2` classes. parser.js flips their `hidden` attribute, and Tailwind's preflight makes `hidden` win over `flex` and similar classes.
- `#go` contains **only** the text `Login`, because parser.js compares `go_button.textContent === "Login"`. Don't put an icon or extra whitespace inside it.

Purely visual additions:

- The fields sit inside `<form id="login-form">`, with `#go` as the submit button. Pressing Enter clicks `#go`, which runs parser.js's existing handler.
- The password field is `type="password"` and has an eye button to show it.
- `#login-msg` turns green for "Login successful" and "Account created…", and stays red for everything else.

**After login:** `ui.js` watches `#login-msg`. When it reads exactly `Login successful`, it calls `load_entries()` and switches to the list. If you ever change that message text, change the check in `ui.js` too.

## Vault actions (bottom of parser.js)

`ui.js` imports these and calls them directly. They're blank: fill them in. Each one returns an **error message** for the UI to show, or **nothing** for success. The UI only updates its list after a call succeeds, so a failed save never looks like it worked.

| Function | Called when | Arguments |
|---|---|---|
| `load_entries()` | Right after login | Should **return** the list `[{ id, title, username, password }]` |
| `save_entry(entry, is_new, next)` | Save on the entry page (edit or add) | `entry` = the login saved, `is_new` = true when added |
| `delete_entries(ids, next)` | Trash button, after the confirm dialog | `ids` = the selected logins' ids |
| `change_hint(hint)` | Account › Save hint | UI already rejects an empty hint |
| `change_password(current, next)` | Account › Change password | UI already checks both are filled and the confirm field matches |
| `lock_vault()` | Lock vault (menu or account page) | The UI clears itself and returns to login afterwards |

`next` is the **whole** list of logins after the change. The vault is encrypted as one blob, so you never have to merge anything: serialize `next`, encrypt it, push it.

New logins get `id = crypto.randomUUID()`. Titles are trimmed. Usernames and passwords are saved exactly as typed, since spaces can be part of a password.

## Filling them in with the current JS and Rust

The stubs live in parser.js, so they can use `vault_id`, `vault_key` and `vault_plaintext` directly.

### Plaintext format, and `load_entries`

Accounts are created with an encrypted empty string, so plaintext `""` means "no logins". Store the list as JSON:

```js
export async function load_entries() {
    return vault_plaintext ? JSON.parse(vault_plaintext) : [];
}
```

### Save and delete use the existing `encrypt_and_push`

`encrypt_and_push(id, plaintext, key)` already exists (add it to the import at the top of parser.js). It returns `"OK"` on success:

```js
async function push_vault(next) {
    const plaintext = JSON.stringify(next);
    const res = await encrypt_and_push(vault_id, plaintext, vault_key);
    if (res !== "OK") return res;
    vault_plaintext = plaintext;
}

export async function save_entry(entry, is_new, next) { return push_vault(next); }
export async function delete_entries(ids, next) { return push_vault(next); }
```

### Lock must wipe the key

```js
export function lock_vault() { vault_key = null; vault_plaintext = ""; vault_id = 0; }
```

### Account actions need new Rust functions

These don't exist yet. Suggested shapes, following the existing style of returning `"OK"` or an error `String`:

- **`update_password_hint(id: i32, hint: String) -> String`**: one `client.update_record_by_id("ironvault", id, "password_hint", …)`.
  ```js
  export async function change_hint(hint) {
      const r = await update_password_hint(vault_id, hint);
      if (r !== "OK") return r;
  }
  ```
- **`change_master_password(id, plaintext, new_password) -> String`**: generate a new salt, derive the new key, re-encrypt `plaintext`, and write salt, ciphertext and nonce. In `change_password`, first check `current` the same way `get_plaintext` does (`get_unwrapped_key(current, kdf_salt)` + `decrypt`). On success, replace `vault_key` with the new key, which the Rust function should return or let JS derive again.

### Backend risk worth knowing (not changed)

`encrypt_and_push` writes `ciphertext` and `nonce` as **two separate updates**. If the first succeeds and the second fails, the stored ciphertext no longer matches its nonce and the vault can't be decrypted. Writing both columns in one `UPDATE` (and salt too, for password changes) would fix it.

## Styling notes

- **Tailwind v4** runs from the in-browser build (`@tailwindcss/browser@4.1.11` on jsDelivr), with theme tokens in the `<style type="text/tailwindcss">` block. That is fine for development. For a hosted password manager, avoid running a third-party script on the page: build with the Tailwind CLI (`npx @tailwindcss/cli -i in.css -o tailwind.css`, moving the `@theme`/`@layer` block into `in.css`), link the output CSS, and delete the CDN script.
- **Icons** are Heroicons v2.2.0 (24px outline), inlined once as `<symbol>`s at the top of `<body>`. Use `<svg class="size-5" aria-hidden="true"><use href="#i-trash"/></svg>`. To add one, copy its path from heroicons.com into a new `<symbol id="i-…">`.
- **Palette** (in `@theme`): `paper` #F8EAD6 (page), `plate` #FFF9F0 (panels), `forge` #EF7D16 (header), `ember` #A9530B (accents and focus), `iron` #2A2420 (text), `slag` #6E6257 (muted text), `seam` #E4CFB0 (borders), `patina` #2E7552 (primary buttons), `rust` #B3362B (errors and delete).
- **Type**: Bitcount Prop Single (pixel) for the wordmark, entry titles, username heading and row initials; Archivo for everything else.
