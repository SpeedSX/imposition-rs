# Build release wasm32 and refresh pkg/ for the Vite web UI (requires wasm-bindgen in PATH).
$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen "target/wasm32-unknown-unknown/release/layout_rs.wasm" --out-dir pkg --target web
