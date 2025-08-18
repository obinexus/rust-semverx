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
