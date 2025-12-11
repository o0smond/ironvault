#!/bin/bash
#for home setup
result=$(which python)
if [ "$result" == "/usr/bin/python" ]; then
    source /home/oliver/py311_venv/bin/activate
fi
result=$(maturin -V)
if [ "$result" != "bash: maturin: command not found" ]; then
    pip install maturin
fi
cd vault_core
maturin develop --release
cd ..
pyinstaller --name PasswordManager --onefile --windowed \
--add-data "gui:gui" \
--add-data "vault_core/src/storage.json:vault_core/src" \
--add-binary "vault_core/target/debug/release/libvault_core.so:vault_core" \
interface/page_1.py