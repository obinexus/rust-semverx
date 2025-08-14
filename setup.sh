#!/bin/bash
# OBINexus rust-semverx Setup Script
# Part of the polyglot toolchain: riftlang.exe → .so.a → rift.exe → gosilang

echo "Setting up rust-semverx for OBINexus..."

# Ensure proper directory structure
mkdir -p src/bin
mkdir -p schemas
mkdir -p scripts

# Initialize cargo project if needed
if [ ! -f "Cargo.toml" ]; then
    echo "Error: Cargo.toml not found"
    exit 1
fi

# Build the project
cargo build --release

echo "rust-semverx setup complete"
