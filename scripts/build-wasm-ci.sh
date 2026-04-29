#!/usr/bin/env bash
# Build wasm32 + wasm-bindgen output for Vite (Linux / Vercel). Windows devs use build-wasm.ps1.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if ! command -v cargo >/dev/null 2>&1; then
  export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
  export RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable
  # shellcheck source=/dev/null
  source "$CARGO_HOME/env"
fi

rustup target add wasm32-unknown-unknown

WBG_VERSION="0.2.120"
if ! command -v wasm-bindgen >/dev/null 2>&1; then
  cargo install wasm-bindgen-cli --version "$WBG_VERSION" --locked --no-default-features
fi

cargo build --release --target wasm32-unknown-unknown
wasm-bindgen "$ROOT/target/wasm32-unknown-unknown/release/layout_rs.wasm" --out-dir "$ROOT/pkg" --target web
