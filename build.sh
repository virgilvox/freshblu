#!/usr/bin/env bash
set -e

echo "=== Building FreshBlu ==="

# Install Rust if not present
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

echo "Building server + CLI..."
cargo build --release --bin freshblu-server --bin freshblu

echo ""
echo "Binaries:"
echo "  ./target/release/freshblu-server"
echo "  ./target/release/freshblu"

# Build WASM if wasm-pack is available
if command -v wasm-pack &> /dev/null; then
    echo ""
    echo "Building WASM..."
    cd crates/freshblu-wasm
    wasm-pack build --target web --out-dir ../../dist/wasm-web
    wasm-pack build --target nodejs --out-dir ../../dist/wasm-node
    cd ../..
    echo "WASM built to dist/wasm-*"
else
    echo ""
    echo "To build WASM: cargo install wasm-pack && ./build.sh"
fi

# Build JS SDK
if command -v npm &> /dev/null; then
    echo ""
    echo "Building JS SDK..."
    cd sdks/js && npm install && npm run build && cd ../..
    echo "JS SDK built to sdks/js/dist/"
fi

echo ""
echo "=== Build complete! ==="
echo ""
echo "Quick start:"
echo "  ./target/release/freshblu-server"
echo "  ./target/release/freshblu status"
