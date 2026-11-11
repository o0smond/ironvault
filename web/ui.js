// UI layer only: switches views, renders the entry list, tracks selection, and calls the
// vault actions in parser.js for every change. See UI_INTEGRATION.md.
import { load_entries, save_entry, delete_entries, change_hint, change_password, lock_vault } from "./parser.js";

const $ = (sel) => document.querySelector(sel);
const views = document.querySelectorAll('[data-view]');
const fields = document.querySelectorAll('#entry-form [data-field]');
const list = $('#entry-list');
const rowTpl = $('#row-tpl');
const toast = $('#toast');
const menu = $('#menu');
const deleteDialog = $('#confirm-delete');
const entryReveal = $('[data-reveal="f-password"]');
const loginReveal = $('[data-reveal="password"]');

const selected = new Set();
let entries = [];
let current = null; // entry open in the entry view, null while adding a new one
let toastTimer;

function showView(name) {
    views.forEach(v => { v.hidden = v.dataset.view !== name; });
    menu.open = false;
    window.scrollTo(0, 0);
}

function setUser(name) {
    document.querySelectorAll('[data-username]').forEach(el => { el.textContent = name; });
}

// entries: [{ id, title, username, password }]
function setEntries(next) {
    entries = [...next];
    for (const id of selected) if (!entries.some(e => e.id === id)) selected.delete(id);
    render();
}

const getSelectedIds = () => [...selected];
const getSelectedEntries = () => entries.filter(e => selected.has(e.id));

function notify(message) {
    toast.textContent = message;
    toast.hidden = false;
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => { toast.hidden = true; }, 4_000);
}

function render() {
    list.replaceChildren(...entries.map(entry => {
        const row = rowTpl.content.firstElementChild.cloneNode(true);
        const box = row.querySelector('input');
        box.checked = selected.has(entry.id);
        box.setAttribute('aria-label', `Select ${entry.title}`);
        box.addEventListener('change', () => {
            box.checked ? selected.add(entry.id) : selected.delete(entry.id);
            syncToolbar();
        });
        row.querySelector('[data-title]').textContent = entry.title;
        row.querySelector('[data-user]').textContent = entry.username;
        row.querySelector('[data-mono]').textContent =
            (entry.title.replace(/^https?:\/\/(www\.)?/i, '')[0] ?? '?').toUpperCase();
        row.querySelector('[data-open]').addEventListener('click', () => openEntry(entry));
        return row;
    }));
    $('#empty').hidden = entries.length > 0;
    syncToolbar();
}

function syncToolbar() {
    const n = selected.size;
    $('#edit-selected').disabled = n !== 1;
    $('#delete-selected').disabled = n === 0;
    $('#selection-count').textContent = n ? `${n} selected` : '';
}

// reveal: a [data-reveal] eye button; its value is the id of the password input it toggles
function setPasswordVisible(reveal, visible) {
    document.getElementById(reveal.dataset.reveal).type = visible ? 'text' : 'password';
    reveal.setAttribute('aria-pressed', String(visible));
    reveal.setAttribute('aria-label', visible ? 'Hide password' : 'Show password');
    reveal.querySelector('use').setAttribute('href', visible ? '#i-eye-slash' : '#i-eye');
}

function openEntry(entry, editAll = false) {
    current = entry;
    fields.forEach(f => {
        f.value = entry?.[f.dataset.field] ?? '';
        f.readOnly = !editAll;
    });
    setPasswordVisible(entryReveal, false);
    $('#entry-actions').hidden = !editAll;
    $('#entry-save').textContent = entry ? 'Save changes' : 'Save login';
    showView('entry');
    if (editAll) fields[0].focus();
}

async function saveEntry() {
    const draft = { id: current?.id ?? crypto.randomUUID() };
    fields.forEach(f => { draft[f.dataset.field] = f.value; }); // never trim: spaces can be part of a password
    draft.title = draft.title.trim();
    if (!draft.title) return notify('Give this login a website or title');

    const isNew = !current;
    const next = isNew ? [...entries, draft] : entries.map(e => (e.id === draft.id ? draft : e));
    const err = await save_entry(draft, isNew, next);
    if (err) return notify(err);
    setEntries(next);
    openEntry(draft);
    notify(isNew ? 'Login saved' : 'Changes saved');
}

async function deleteSelected() {
    const ids = getSelectedIds();
    const next = entries.filter(e => !selected.has(e.id));
    const err = await delete_entries(ids, next);
    if (err) return notify(err);
    setEntries(next);
    notify(ids.length === 1 ? 'Login deleted' : `${ids.length} logins deleted`);
}

async function lock() {
    await lock_vault();
    selected.clear();
    setEntries([]);
    setUser('');
    document.querySelectorAll('#password, [data-view="account"] input').forEach(i => { i.value = ''; });
    setPasswordVisible(loginReveal, false);
    showView('login');
}

// ---- Wiring ---------------------------------------------------------------------------------

// Enter submits via #go's click, which is what parser.js listens for.
$('#login-form').addEventListener('submit', e => e.preventDefault());

// shortcut: parser.js has no login-success hook, so watch the message it writes; a direct call
// from get_plaintext would be sturdier if that message text ever changes.
const loginMsg = $('#login-msg');
new MutationObserver(async () => {
    const text = loginMsg.textContent;
    loginMsg.dataset.tone = /successful|created/i.test(text) ? 'ok' : 'error';
    if (text === 'Login successful') {
        setUser($('#username').value);
        $('#password').value = '';
        setPasswordVisible(loginReveal, false);
        setEntries((await load_entries()) ?? []);
        showView('list');
    }
}).observe(loginMsg, { childList: true, characterData: true, subtree: true });

document.querySelectorAll('[data-go]').forEach(b => b.addEventListener('click', () => showView(b.dataset.go)));
document.querySelectorAll('[data-action="add"]').forEach(b => b.addEventListener('click', () => openEntry(null, true)));
document.querySelectorAll('[data-action="lock"]').forEach(b => b.addEventListener('click', lock));

document.addEventListener('click', e => {
    if (!e.target.closest('#menu summary')) menu.open = false;
});
document.addEventListener('keydown', e => {
    if (e.key === 'Escape') menu.open = false;
});

$('#edit-selected').addEventListener('click', () => openEntry(getSelectedEntries()[0], true));

$('#delete-selected').addEventListener('click', () => {
    const n = selected.size;
    $('#delete-title').textContent = n === 1 ? `Delete ${getSelectedEntries()[0].title}?` : `Delete ${n} logins?`;
    $('#delete-confirm').textContent = n === 1 ? 'Delete login' : `Delete ${n} logins`;
    deleteDialog.showModal();
});
$('#delete-confirm').addEventListener('click', deleteSelected);

document.querySelectorAll('[data-edit]').forEach(b => b.addEventListener('click', () => {
    const f = $(`#entry-form [data-field="${b.dataset.edit}"]`);
    f.readOnly = false;
    f.focus();
    $('#entry-actions').hidden = false;
}));
document.querySelectorAll('[data-reveal]').forEach(b => b.addEventListener('click', () =>
    setPasswordVisible(b, b.getAttribute('aria-pressed') !== 'true')));
$('#entry-cancel').addEventListener('click', () => (current ? openEntry(current) : showView('list')));
$('#entry-form').addEventListener('submit', e => {
    e.preventDefault();
    saveEntry();
});

$('#hint-form').addEventListener('submit', async e => {
    e.preventDefault();
    const hint = $('#hint').value.trim();
    if (!hint) return notify('Enter a password hint');
    const err = await change_hint(hint);
    notify(err || 'Password hint saved');
});

$('#password-form').addEventListener('submit', async e => {
    e.preventDefault();
    const current = $('#pw-current').value;
    const next = $('#pw-next').value;
    if (!current || !next) return notify('Enter your current and new password');
    if (next !== $('#pw-confirm').value) return notify("New passwords don't match");
    const err = await change_password(current, next);
    if (err) return notify(err);
    e.target.reset();
    notify('Master password changed');
});
