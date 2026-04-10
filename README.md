# layout-rs

Proportion and sheet-planning logic, exposed as a Rust library and compiled to WebAssembly for the bundled web UI.

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- WebAssembly target: `rustup target add wasm32-unknown-unknown`
- [wasm-bindgen](https://rustwasm.github.io/wasm-bindgen/) CLI on your `PATH` (e.g. `cargo install wasm-bindgen-cli`)
- [Node.js](https://nodejs.org/) (for the Vite frontend)

## Rust library and tests

```powershell
cargo build
cargo test
```

## Web UI (WASM + Vite)

From the repo root:

```powershell
npm install
npm run wasm:build
npm run dev
```

`wasm:build` runs `build-wasm.ps1`, which produces `pkg/` for the app. The dev server defaults to port **5174** and may open the browser automatically.

Static production build and local preview:

```powershell
npm run wasm:build
npm run build
npm run preview
```
