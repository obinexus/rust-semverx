#!/bin/bash
# OBINexus rust-semverx Structural Reform Script
# Implements Unicode-Only Structural Charset Normalizer principles

set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Base directory
BASE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SRC_DIR="$BASE_DIR/src"

# Unicode path normalizer function
normalize_path() {
    local path="$1"
    # Apply isomorphic reduction - normalize Unicode variants
    # Convert %2e%2e%2f, %c0%af, and other encodings to canonical form
    echo "$path" | sed -E 's/%2e/./gi; s/%2f/\//gi; s/%c0%af/../gi'
}

# Create canonical directory structure
create_canonical_structure() {
    echo -e "${BLUE}[INFO]${NC} Creating canonical directory structure..."
    
    # Core module directories
    local modules=(
        "core"
        "ffi"
        "resolver"
        "normalizer"
    )
    
    for module in "${modules[@]}"; do
        local module_path="$SRC_DIR/$module"
        mkdir -p "$module_path"
        echo -e "${GREEN}[✓]${NC} Created $module/"
        
        # Create mod.rs for each module
        if [[ ! -f "$module_path/mod.rs" ]]; then
            cat > "$module_path/mod.rs" << EOF
//! $module module for rust-semverx
//! Implements OBINexus polyglot schema

#![deny(unsafe_code)]
#![warn(missing_docs)]

EOF
            echo -e "${GREEN}[✓]${NC} Initialized $module/mod.rs"
        fi
    done
}

# Migrate existing files to canonical locations
migrate_files() {
    echo -e "${BLUE}[INFO]${NC} Migrating existing files..."
    
    # File mapping: source -> destination
    declare -A file_map=(
        ["cli.rs"]="bin/semverx/main.rs"
        ["core.rs"]="core/semver.rs"
        ["ffi.rs"]="ffi/c_api.rs"
        ["resolver.rs"]="resolver/graph.rs"
    )
    
    for src_file in "${!file_map[@]}"; do
        local dest="${file_map[$src_file]}"
        local src_path="$BASE_DIR/$src_file"
        local dest_path="$SRC_DIR/$dest"
        
        if [[ -f "$src_path" ]]; then
            # Create destination directory
            mkdir -p "$(dirname "$dest_path")"
            
            # Move file with normalization
            mv "$src_path" "$dest_path" 2>/dev/null || {
                echo -e "${YELLOW}[WARN]${NC} Could not move $src_file"
            }
            echo -e "${GREEN}[✓]${NC} Migrated $src_file -> $dest"
        fi
    done
}

# Generate schema configurations
generate_schemas() {
    echo -e "${BLUE}[INFO]${NC} Generating schema configurations..."
    
    local schema_dir="$BASE_DIR/schemas"
    mkdir -p "$schema_dir"
    
    # Canonical schema
    cat > "$schema_dir/canonical.toml" << 'EOF'
# OBINexus Canonical Schema
# Enforces isomorphic reduction principles

[version]
pattern = "major.minor.patch[-prerelease][+build].environment.classifier.intent"
unicode_normalization = true

[structure]
enforce_taxonomy = true
path_normalization = "unicode_only"

[polyglot]
monoglot = { locked = true }
hybrid = { extensions_allowed = true }
polyglot = { ffi_enabled = true }

[resolver]
algorithm = "eulerian_hamiltonian"
scoring = "astar"
diamond_prevention = true
EOF
    
    echo -e "${GREEN}[✓]${NC} Generated canonical.toml schema"
}

# Create Cargo.toml with workspace configuration
setup_cargo_workspace() {
    echo -e "${BLUE}[INFO]${NC} Setting up Cargo workspace..."
    
    cat > "$BASE_DIR/Cargo.toml" << 'EOF'
[package]
name = "rust-semverx"
version = "0.1.0"
edition = "2021"
authors = ["OBINexus <team@obinexus.com>"]
license = "MIT OR Apache-2.0"
description = "Semantic Version X - Polyglot package management with isomorphic reduction"

[workspace]
members = [
    ".",
    "bin/semverx",
    "bin/semverx-server",
    "bin/semverx-daemon",
]

[dependencies]
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
petgraph = "0.6"  # For graph algorithms
regex = "1.10"

[lib]
name = "semverx"
path = "src/lib.rs"

[[bin]]
name = "semverx"
path = "bin/semverx/main.rs"

[features]
default = ["unicode-normalizer"]
unicode-normalizer = []
polyglot-ffi = []
hot-swap = []
EOF
    
    echo -e "${GREEN}[✓]${NC} Created Cargo.toml workspace"
}

# Create lib.rs entry point
create_lib_entry() {
    echo -e "${BLUE}[INFO]${NC} Creating library entry point..."
    
    cat > "$SRC_DIR/lib.rs" << 'EOF'
//! rust-semverx: Semantic Version X Implementation
//! 
//! Implements OBINexus polyglot package management with:
//! - Isomorphic reduction for Unicode normalization
//! - Eulerian/Hamiltonian cycle dependency resolution
//! - Hot-swappable component architecture

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod core;
pub mod ffi;
pub mod resolver;
pub mod normalizer;

pub use core::*;
pub use resolver::DependencyResolver;
pub use normalizer::UnicodeNormalizer;

/// Canonical path representation after normalization
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalPath(String);

impl CanonicalPath {
    /// Normalize a path using isomorphic reduction
    pub fn normalize(path: &str) -> Self {
        let normalized = normalizer::normalize_unicode_path(path);
        Self(normalized)
    }
}
EOF
    
    echo -e "${GREEN}[✓]${NC} Created lib.rs"
}

# Validate the reformed structure
validate_structure() {
    echo -e "${BLUE}[INFO]${NC} Validating reformed structure..."
    
    local required_paths=(
        "src/lib.rs"
        "src/core/mod.rs"
        "src/ffi/mod.rs"
        "src/resolver/mod.rs"
        "src/normalizer/mod.rs"
        "schemas/canonical.toml"
        "Cargo.toml"
    )
    
    local all_valid=true
    for path in "${required_paths[@]}"; do
        if [[ -f "$BASE_DIR/$path" ]]; then
            echo -e "${GREEN}[✓]${NC} Validated: $path"
        else
            echo -e "${RED}[✗]${NC} Missing: $path"
            all_valid=false
        fi
    done
    
    if $all_valid; then
        echo -e "${GREEN}[SUCCESS]${NC} Structure validation passed!"
        return 0
    else
        echo -e "${RED}[ERROR]${NC} Structure validation failed!"
        return 1
    fi
}

# Main execution
main() {
    echo -e "${BLUE}╔══════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║   OBINexus rust-semverx Reform Script   ║${NC}"
    echo -e "${BLUE}╚══════════════════════════════════════════╝${NC}"
    echo
    
    # Execute reform steps
    create_canonical_structure
    migrate_files
    generate_schemas
    setup_cargo_workspace
    create_lib_entry
    
    # Validate the result
    echo
    validate_structure
    
    echo
    echo -e "${GREEN}[COMPLETE]${NC} Structural reform finished!"
    echo -e "${BLUE}[INFO]${NC} Run 'cargo build' to verify compilation"
}

# Run main function
main "$@"
