#!/bin/bash
set -e

# Install Rust if not present
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

echo "Rust $(rustc --version)"

# Download and compile all dependencies
cargo build

echo "Done. Run with: cargo run -p engine -- --watch <path> --typescript"
