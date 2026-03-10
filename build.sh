#!/usr/bin/env bash
set -euo pipefail

echo "==> Building WASM with wasm-pack..."
wasm-pack build --target web --out-dir web/pkg --release

echo "==> Build complete. Output in web/"
