# IronVault audit: what's left

Last checked 2026-11-13 against the uncommitted working tree (branch `web-edition`). Each item was checked against the code. Items marked *(unverified)* are inferred and need a live test.

## 0. Do first: test against the live DB

Nothing below has been run against the real worker and Turso. Make a throwaway account and check:

- [ ] Create account, log in, wrong password, unknown user.
- [ ] Add, edit and delete an entry, then log out and back in. The list should match.
- [ ] Change the hint, log out and back in. The DB row should have the new hint.

The code paths were checked against `worker/worker.js`. Its `update` action writes every column in one `UPDATE` and returns `affected_row_count`, which is what `encrypt_and_push` expects.

## 1. High

### The worker has no auth
`worker/worker.js` accepts any POST from any origin (`Access-Control-Allow-Origin: *`) with no key or session. Anyone who finds the URL (it's in the shipped wasm) can:
- `fetch`: dump every user's ciphertext, salt and hint for offline cracking.
- `update` / `update_by_column`: overwrite any row's ciphertext, which destroys that user's vault.
- `delete`: delete any account.
- Pass any `table` name, so the whole database is open, not just `ironvault`.

Smallest useful fix:
- Remove the `fetch` and `delete` actions, since the app doesn't use them.
- Hard-code the table name in the worker.
- Only allow the columns the app writes: `ciphertext`, `nonce`, `password_hint`, and later `kdf_salt`.

Real fix: make writes prove they know the key. For example, store a hash of an auth key derived from the master key, and have the worker check it.

### Master password change isn't built
`change_password` in `web/parser.js` returns "isn't built yet". Steps, all in JS except key derivation and encryption:
1. Keep `kdf_salt` from `fetch_db_info` in parser.js. It's thrown away today.
2. Check `current`: `get_unwrapped_key(current, kdf_salt)` must equal `vault_key` byte for byte.
3. Make a new salt in JS (`crypto.getRandomValues`, then base64).
4. `new_key = get_unwrapped_key(next, new_salt)`.
5. Encrypt `entries` with `new_key` and write ciphertext, nonce, hint **and `kdf_salt` in one `update`**. If the salt is written separately and fails, no password can open the vault. That means `encrypt_and_push` needs an optional salt arg, or a sibling function.
6. On success, set `vault_key = new_key`.

## 2. Medium

- [ ] **The hint is never shown.** There's no "forgot password" path on the login screen, and the account page's `#hint` input isn't prefilled. A hint can be saved but never read.
- [ ] **`decrypt` panics on a bad nonce length.** `XNonce::from_slice` panics on a nonce that isn't 24 bytes (`rust_core/src/lib.rs`, `decrypt`). JS catches the trap, but the wasm instance may be unusable afterwards. Check the length and return an error string instead.
- [ ] **Errors are matched by string.** `decrypt` returns `"decryption error"` and similar in place of the plaintext, and parser.js compares strings. It's safe today because the plaintext is always JSON, but it breaks if any message text changes.
- [ ] **Duplicate usernames** *(unverified)*. Create account does `login_check`, then `insert`, and the worker's `insert` doesn't check for an existing name. Two people signing up at once can both pass the check, and only a `UNIQUE` constraint on `username` in Turso stops that. Check the schema. If two rows ever share a name, `fetch_by_user` silently logs into the first one.
- [ ] **No auto-lock.** `vault_key` and the decrypted `entries` stay in memory until the user clicks Lock. Add an idle timer that calls `lock()`.

## 3. Low

- [ ] **Build warnings.** `use crate::utils::database::Entry;` in `lib.rs` is now unused (`add_new_entry` was removed), so `Entry` in `database.rs` is dead code. Delete both, or keep them if Rust will read entries later.
- [ ] **Unused crates.** `rand_core` and `chacha20` in `Cargo.toml` are never imported.
- [ ] **No way out of create-account mode.** The `.s2` view has no cancel or back button, so the user has to reload.
- [ ] **Bad vault data crashes render.** If the decrypted JSON isn't an array of `{ id, title, … }`, `render()` throws and the user is stuck on "Login successful". Check `Array.isArray` in `get_plaintext`.
- [ ] **Messages linger after lock.** "Login successful" and toasts stay up to 5s after locking. Clear `#login-msg` in `lock()`.
- [ ] **`crypto.randomUUID` needs a secure context.** It only works over HTTPS or on localhost. Fine once hosted on HTTPS.
- [ ] **Argon2 params aren't stored per user.** 64 MiB, 3 passes and 1 lane are hard-coded, so they can't be raised later without breaking old accounts. Store them in the row before there are real users.
- [ ] **Google Fonts.** It's CSS only, so it can't run code, but Google sees every visit. Self-host the fonts for zero outside requests.

## Already fixed (this session, uncommitted)

- Ciphertext, nonce and hint are saved in **one** DB write, so a vault can't be half-written.
- `Ok(0)` rows updated now reports an error instead of `"OK"`.
- Double-clicking Save can't lose an entry. `push_vault` refuses a second push while one is running.
- Locking during a save no longer refills the vault.
- The password-change stub no longer shows "Master password changed".
- Lock clears the entry form, including a shown password.
- Tailwind is built ahead of time (`npm run build:css`), so no third-party script runs on the page.
- parser.js owns all vault state, and ui.js only renders it.
