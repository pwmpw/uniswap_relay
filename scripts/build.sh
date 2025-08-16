#!/bin/bash

# Uniswap Relay DApp Build Script
set -e

echo "🚀 Building Uniswap Relay DApp..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Please install Rust first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# Check Rust version
RUST_VERSION=$(rustc --version | cut -d' ' -f2 | cut -d'.' -f1-2)
REQUIRED_VERSION="1.75"

if [ "$(printf '%s\n' "$REQUIRED_VERSION" "$RUST_VERSION" | sort -V | head -n1)" != "$REQUIRED_VERSION" ]; then
    echo "❌ Rust version $RUST_VERSION is too old. Required: $REQUIRED_VERSION+"
    echo "   Please update Rust: rustup update"
    exit 1
fi

echo "✅ Rust version: $RUST_VERSION"

# Clean previous builds
echo "🧹 Cleaning previous builds..."
cargo clean

# Check dependencies
echo "📦 Checking dependencies..."
cargo check

# Run tests
echo "🧪 Running tests..."
cargo test

# Build in release mode
echo "🔨 Building in release mode..."
cargo build --release

# Check binary size
BINARY_SIZE=$(du -h target/release/uniswap_relay | cut -f1)
echo "📊 Binary size: $BINARY_SIZE"

# Run clippy for code quality
echo "🔍 Running clippy..."
cargo clippy -- -D warnings

# Security audit
echo "🔒 Running security audit..."
cargo audit

echo "✅ Build completed successfully!"
echo "📁 Binary location: target/release/uniswap_relay"
echo "🚀 Run with: ./target/release/uniswap_relay"