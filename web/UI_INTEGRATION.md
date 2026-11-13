# IronVault web UI — how it works and how to wire it up

| File | What it is |
|---|---|
| `index.html` | All four views as markup, and the Heroicons sprite |
| `tailwind.in.css` | The Tailwind theme and component classes. Source for `tailwind.css` |
| `tailwind.css` | Built CSS the page loads. Don't edit by hand |
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

**After login:** once the vault decrypts, `get_plaintext` in parser.js fires a `document` event named `unlocked`. `ui.js` listens for it, renders the list and switches to it.

## Vault actions (bottom of parser.js)

parser.js owns all vault state: `vault_id`, `vault_key` and `entries` (the decrypted list, `[{ id, title, username, password }]`). `ui.js` never keeps its own copy. It calls these, then calls `render()`, which reads `get_entries()` again.

Each action returns an **error message** for the UI to show, or **nothing** for success. `entries` only changes after `encrypt_and_push` returns `"OK"`, so a failed save never looks like it worked.

| Function | Called when | Does |
|---|---|---|
| `get_entries()` | Every render | Returns the list |
| `save_entry(entry)` | Save on the entry page (edit or add) | Replaces the entry with the same `id`, or adds it, then encrypts and pushes |
| `delete_entries(ids)` | Trash button, after the confirm dialog | Removes those ids, then encrypts and pushes |
| `change_hint(hint)` | Account › Save hint | Pushes the vault again with the new hint. UI already rejects an empty hint |
| `change_password(current, next)` | Account › Change password | Stub. UI already checks both are filled and the confirm field matches |
| `lock_vault()` | Lock vault (menu or account page) | Wipes `vault_id`, `vault_key`, `entries` and `password_hint`. The UI then returns to login |

The plaintext is `JSON.stringify(entries)`. Accounts are created with an encrypted empty string, so `""` means "no logins". The vault is one encrypted blob, so every change re-encrypts the whole list.

New logins get `id = crypto.randomUUID()` in ui.js. Titles are trimmed. Usernames and passwords are saved exactly as typed, since spaces can be part of a password.

The hint is not encrypted. It stays in its own `password_hint` column, so it can be shown before login. parser.js keeps it in `password_hint`, and `encrypt_and_push(id, plaintext, key, hint)` writes it along with the ciphertext and nonce.

### Changing the master password (not built yet)

JS runs the steps. Rust only derives the key, encrypts and pushes:

1. Check `current`: `get_unwrapped_key(current, kdf_salt)` must match `vault_key` byte for byte. parser.js will need to keep `kdf_salt` from `fetch_db_info`.
2. Make a fresh salt in JS (`crypto.getRandomValues`, then base64).
3. `new_key = get_unwrapped_key(next, new_salt)`.
4. Encrypt and push `entries` with `new_key`, **and write the new salt in the same request**.
5. Set `vault_key = new_key`.

### Backend risk worth knowing (not changed)

`encrypt_and_push` writes `ciphertext`, `nonce` and `password_hint` as **three separate updates**. If the ciphertext saves but the nonce fails, they no longer match and the vault can't be decrypted. A failed hint write only leaves the old hint. For password changes this gets worse: new ciphertext with the old salt can't be opened by any password. Writing every column in one request (`update_record_by_struct` in database.rs, if the worker supports its `update` action) would fix it.

## Styling notes

- **Tailwind v4** is built ahead of time, so no third-party script runs on the page. Theme tokens and component classes live in `tailwind.in.css`. Classes are only generated for what the CLI finds in `web/`, so **after adding or changing classes, run `npm run build:css` from the repo root** (first time: `npm install`). The output `tailwind.css` is committed, so serving `web/` needs no build.
- **Icons** are Heroicons v2.2.0 (24px outline), inlined once as `<symbol>`s at the top of `<body>`. Use `<svg class="size-5" aria-hidden="true"><use href="#i-trash"/></svg>`. To add one, copy its path from heroicons.com into a new `<symbol id="i-…">`.
- **Palette** (in `@theme`): `paper` #F8EAD6 (page), `plate` #FFF9F0 (panels), `forge` #EF7D16 (header), `ember` #A9530B (accents and focus), `iron` #2A2420 (text), `slag` #6E6257 (muted text), `seam` #E4CFB0 (borders), `patina` #2E7552 (primary buttons), `rust` #B3362B (errors and delete).
- **Type**: Bitcount Prop Single (pixel) for the wordmark, entry titles, username heading and row initials; Archivo for everything else.
