#!/usr/bin/env bash
# Build rust_core to wasm and replace web/pkg with the fresh build.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# --target web: parser.js imports pkg/ironvault.js as a native ES module
wasm-pack build "$root/rust_core" --target web --release

rm -rf "$root/web/pkg"
mv "$root/rust_core/pkg" "$root/web/pkg"
echo "Built and moved pkg to $root/web/pkg"
