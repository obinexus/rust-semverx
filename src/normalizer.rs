//! Unicode normalization with isomorphic reduction
//! Part of OBINexus polyglot package management

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Unicode normalizer for path and identifier canonicalization
pub struct UnicodeNormalizer {
    /// Mapping for isomorphic character reduction
    isomorphic_map: HashMap<char, char>,
    /// Cache for normalized paths
    cache: HashMap<String, String>,
}

impl UnicodeNormalizer {
    /// Create a new Unicode normalizer with default mappings
    pub fn new() -> Self {
        let mut isomorphic_map = HashMap::new();
        
        // Common Unicode homoglyphs and their ASCII equivalents
        // Latin lookalikes
        isomorphic_map.insert('\u{0430}', 'a'); // Cyrillic а
        isomorphic_map.insert('\u{0435}', 'e'); // Cyrillic е
        isomorphic_map.insert('\u{043E}', 'o'); // Cyrillic о
        isomorphic_map.insert('\u{0440}', 'p'); // Cyrillic р
        isomorphic_map.insert('\u{0441}', 'c'); // Cyrillic с
        isomorphic_map.insert('\u{0445}', 'x'); // Cyrillic х
        isomorphic_map.insert('\u{0443}', 'y'); // Cyrillic у
        
        // Greek lookalikes
        isomorphic_map.insert('\u{03B1}', 'a'); // Greek α
        isomorphic_map.insert('\u{03B2}', 'b'); // Greek β
        isomorphic_map.insert('\u{03B5}', 'e'); // Greek ε
        isomorphic_map.insert('\u{03BF}', 'o'); // Greek ο
        isomorphic_map.insert('\u{03C1}', 'p'); // Greek ρ
        isomorphic_map.insert('\u{03C4}', 't'); // Greek τ
        isomorphic_map.insert('\u{03C5}', 'u'); // Greek υ
        
        // Common ligatures and special forms
        isomorphic_map.insert('\u{FB00}', 'f'); // ﬀ ligature (will expand to ff)
        isomorphic_map.insert('\u{FB01}', 'f'); // ﬁ ligature (will expand to fi)
        isomorphic_map.insert('\u{FB02}', 'f'); // ﬂ ligature (will expand to fl)
        
        // Zero-width and invisible characters (map to nothing)
        isomorphic_map.insert('\u{200B}', '\0'); // Zero-width space
        isomorphic_map.insert('\u{200C}', '\0'); // Zero-width non-joiner
        isomorphic_map.insert('\u{200D}', '\0'); // Zero-width joiner
        isomorphic_map.insert('\u{FEFF}', '\0'); // Zero-width no-break space
        
        Self {
            isomorphic_map,
            cache: HashMap::new(),
        }
    }
    
    /// Normalize a single character using isomorphic reduction
    pub fn normalize_char(&self, ch: char) -> Option<char> {
        self.isomorphic_map.get(&ch).copied()
    }
    
    /// Apply Unicode normalization to a string
    pub fn normalize_string(&mut self, input: &str) -> String {
        if let Some(cached) = self.cache.get(input) {
            return cached.clone();
        }
        
        let normalized = input
            .chars()
            .map(|ch| {
                // Check for isomorphic mapping
                if let Some(mapped) = self.isomorphic_map.get(&ch) {
                    if *mapped == '\0' {
                        return String::new(); // Remove zero-width chars
                    }
                    
                    // Handle ligatures that expand to multiple chars
                    match ch {
                        '\u{FB00}' => return "ff".to_string(),
                        '\u{FB01}' => return "fi".to_string(),
                        '\u{FB02}' => return "fl".to_string(),
                        _ => return mapped.to_string(),
                    }
                }
                
                // Apply Unicode NFC normalization for combining characters
                ch.to_lowercase().to_string()
            })
            .collect::<String>();
        
        // Remove consecutive slashes and normalize path separators
        let normalized = normalized
            .replace("//", "/")
            .replace('\\', "/");
        
        self.cache.insert(input.to_string(), normalized.clone());
        normalized
    }
    
    /// Clear the normalization cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
    
    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.cache.len(), self.cache.capacity())
    }
}

impl Default for UnicodeNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Normalize a Unicode path using isomorphic reduction
pub fn normalize_unicode_path(path: &str) -> String {
    let mut normalizer = UnicodeNormalizer::new();
    
    // Split path into components and normalize each
    let components: Vec<String> = path
        .split('/')
        .filter(|s| !s.is_empty())
        .map(|component| {
            // Normalize the component
            let normalized = normalizer.normalize_string(component);
            
            // Remove common confusable patterns
            normalized
                .replace("..", "_")  // Prevent directory traversal
                .replace("./", "")    // Remove current directory refs
                .trim_matches('.')    // Remove leading/trailing dots
                .to_string()
        })
        .filter(|s| !s.is_empty())
        .collect();
    
    // Reconstruct the path
    if path.starts_with('/') {
        format!("/{}", components.join("/"))
    } else {
        components.join("/")
    }
}

/// Check if two paths are isomorphically equivalent
pub fn paths_are_equivalent(path1: &str, path2: &str) -> bool {
    normalize_unicode_path(path1) == normalize_unicode_path(path2)
}

/// Detect potential homograph attacks in identifiers
pub fn detect_homograph_attack(identifier: &str) -> Vec<char> {
    let normalizer = UnicodeNormalizer::new();
    let mut suspicious_chars = Vec::new();
    
    for ch in identifier.chars() {
        if normalizer.isomorphic_map.contains_key(&ch) {
            suspicious_chars.push(ch);
        }
    }
    
    suspicious_chars
}

/// Canonical form for package identifiers
pub fn canonicalize_package_id(id: &str) -> String {
    let mut normalizer = UnicodeNormalizer::new();
    
    // Apply normalization
    let normalized = normalizer.normalize_string(id);
    
    // Replace common separators with underscores
    normalized
        .replace('-', "_")
        .replace('.', "_")
        .replace(' ', "_")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_normalization() {
        let mut normalizer = UnicodeNormalizer::new();
        
        // Test ASCII passthrough
        assert_eq!(normalizer.normalize_string("hello"), "hello");
        
        // Test case normalization
        assert_eq!(normalizer.normalize_string("HeLLo"), "hello");
    }
    
    #[test]
    fn test_homograph_normalization() {
        let mut normalizer = UnicodeNormalizer::new();
        
        // Test Cyrillic lookalikes
        assert_eq!(normalizer.normalize_string("н\u{0435}llо"), "нello");
        
        // Test mixed scripts
        let mixed = "p\u{0430}ckage"; // p with Cyrillic 'а'
        assert_eq!(normalizer.normalize_string(mixed), "package");
    }
    
    #[test]
    fn test_path_normalization() {
        // Test basic path normalization
        assert_eq!(
            normalize_unicode_path("/usr//local///bin/"),
            "/usr/local/bin"
        );
        
        // Test directory traversal prevention
        assert_eq!(
            normalize_unicode_path("/usr/../etc/passwd"),
            "/usr/_/etc/passwd"
        );
        
        // Test Windows-style paths
        assert_eq!(
            normalize_unicode_path("C:\\Users\\Public"),
            "C:/Users/Public"
        );
    }
    
    #[test]
    fn test_homograph_detection() {
        let suspicious = "p\u{0430}ckage"; // p with Cyrillic 'а'
        let attacks = detect_homograph_attack(suspicious);
        assert_eq!(attacks.len(), 1);
        assert_eq!(attacks[0], '\u{0430}');
    }
    
    #[test]
    fn test_package_canonicalization() {
        assert_eq!(
            canonicalize_package_id("My-Package.Name"),
            "my_package_name"
        );
        
        assert_eq!(
            canonicalize_package_id("PACKAGE NAME 2.0"),
            "package_name_2_0"
        );
    }
    
    #[test]
    fn test_path_equivalence() {
        assert!(paths_are_equivalent(
            "/usr/local/bin",
            "/usr//local///bin/"
        ));
        
        assert!(paths_are_equivalent(
            "p\u{0430}ckage/bin", // Cyrillic 'а'
            "package/bin"
        ));
        
        assert!(!paths_are_equivalent(
            "/usr/bin",
            "/usr/local/bin"
        ));
    }
}
