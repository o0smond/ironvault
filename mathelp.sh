#!/bin/bash
result=$(which python)
if [ "$result" == "/usr/bin/python" ]; then
    source /home/oliver/py311_venv/bin/activate
fi
result=$(maturin -V)
if [ "$result" != "bash: maturin: command not found" ]; then
    pip install maturin
fi
maturin develop --manifest-path vault_core/Cargo.toml