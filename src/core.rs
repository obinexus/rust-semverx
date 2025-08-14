//! Core types and traits for rust-semverx
//! OBINexus polyglot package management foundation

use std::fmt;
use std::str::FromStr;

/// Semantic version with extended metadata support
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SemverX {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub prerelease: Option<String>,
    pub build_metadata: Option<String>,
    pub polyglot_hints: Vec<String>,
}

impl SemverX {
    /// Create a new SemverX version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            prerelease: None,
            build_metadata: None,
            polyglot_hints: Vec::new(),
        }
    }
    
    /// Add a polyglot hint for cross-language compatibility
    pub fn add_polyglot_hint(&mut self, hint: String) {
        self.polyglot_hints.push(hint);
    }
    
    /// Check if version satisfies a constraint
    pub fn satisfies(&self, constraint: &VersionConstraint) -> bool {
        match constraint {
            VersionConstraint::Exact(v) => self == v,
            VersionConstraint::GreaterThan(v) => self > v,
            VersionConstraint::LessThan(v) => self < v,
            VersionConstraint::Range(min, max) => self >= min && self <= max,
            VersionConstraint::Compatible(v) => {
                self.major == v.major && self >= v
            }
        }
    }
    
    /// Check if this is a pre-release version
    pub fn is_prerelease(&self) -> bool {
        self.prerelease.is_some()
    }
    
    /// Get the base version string (without metadata)
    pub fn base_version(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl fmt::Display for SemverX {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        
        if let Some(ref pre) = self.prerelease {
            write!(f, "-{}", pre)?;
        }
        
        if let Some(ref build) = self.build_metadata {
            write!(f, "+{}", build)?;
        }
        
        if !self.polyglot_hints.is_empty() {
            write!(f, " [{}]", self.polyglot_hints.join(", "))?;
        }
        
        Ok(())
    }
}

impl FromStr for SemverX {
    type Err = VersionParseError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Simple parser for demonstration
        let parts: Vec<&str> = s.split('.').collect();
        
        if parts.len() < 3 {
            return Err(VersionParseError::InvalidFormat);
        }
        
        let major = parts[0].parse().map_err(|_| VersionParseError::InvalidMajor)?;
        let minor = parts[1].parse().map_err(|_| VersionParseError::InvalidMinor)?;
        
        // Handle patch with optional prerelease/metadata
        let patch_part = parts[2];
        let (patch_str, remainder) = if let Some(idx) = patch_part.find('-') {
            patch_part.split_at(idx)
        } else if let Some(idx) = patch_part.find('+') {
            patch_part.split_at(idx)
        } else {
            (patch_part, "")
        };
        
        let patch = patch_str.parse().map_err(|_| VersionParseError::InvalidPatch)?;
        
        let mut version = SemverX::new(major, minor, patch);
        
        // Parse prerelease and metadata from remainder
        if !remainder.is_empty() {
            if remainder.starts_with('-') {
                if let Some(idx) = remainder[1..].find('+') {
                    version.prerelease = Some(remainder[1..idx+1].to_string());
                    version.build_metadata = Some(remainder[idx+2..].to_string());
                } else {
                    version.prerelease = Some(remainder[1..].to_string());
                }
            } else if remainder.starts_with('+') {
                version.build_metadata = Some(remainder[1..].to_string());
            }
        }
        
        Ok(version)
    }
}

/// Version constraint for dependency resolution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VersionConstraint {
    Exact(SemverX),
    GreaterThan(SemverX),
    LessThan(SemverX),
    Range(SemverX, SemverX),
    Compatible(SemverX), // Caret constraint (^)
}

impl VersionConstraint {
    /// Parse a constraint string (e.g., "^1.2.3", ">2.0.0", "1.0.0 - 2.0.0")
    pub fn parse(s: &str) -> Result<Self, VersionParseError> {
        let s = s.trim();
        
        if s.starts_with('^') {
            let version = SemverX::from_str(&s[1..])?;
            Ok(VersionConstraint::Compatible(version))
        } else if s.starts_with('>') {
            let version = SemverX::from_str(&s[1..].trim())?;
            Ok(VersionConstraint::GreaterThan(version))
        } else if s.starts_with('<') {
            let version = SemverX::from_str(&s[1..].trim())?;
            Ok(VersionConstraint::LessThan(version))
        } else if s.contains(" - ") {
            let parts: Vec<&str> = s.split(" - ").collect();
            if parts.len() != 2 {
                return Err(VersionParseError::InvalidFormat);
            }
            let min = SemverX::from_str(parts[0])?;
            let max = SemverX::from_str(parts[1])?;
            Ok(VersionConstraint::Range(min, max))
        } else {
            let version = SemverX::from_str(s)?;
            Ok(VersionConstraint::Exact(version))
        }
    }
}

/// Error type for version parsing
#[derive(Debug, Clone)]
pub enum VersionParseError {
    InvalidFormat,
    InvalidMajor,
    InvalidMinor,
    InvalidPatch,
}

impl fmt::Display for VersionParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VersionParseError::InvalidFormat => write!(f, "Invalid version format"),
            VersionParseError::InvalidMajor => write!(f, "Invalid major version"),
            VersionParseError::InvalidMinor => write!(f, "Invalid minor version"),
            VersionParseError::InvalidPatch => write!(f, "Invalid patch version"),
        }
    }
}

impl std::error::Error for VersionParseError {}

/// Package metadata for polyglot support
#[derive(Debug, Clone)]
pub struct PackageMetadata {
    pub name: String,
    pub version: SemverX,
    pub language: Language,
    pub build_system: BuildSystem,
    pub dependencies: Vec<Dependency>,
}

/// Supported programming languages in OBINexus
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Language {
    Rust,
    Go,
    Python,
    JavaScript,
    TypeScript,
    C,
    Cpp,
    Java,
    Kotlin,
    Swift,
    Rift,     // OBINexus riftlang
    Gosilang, // OBINexus gosilang
    Other(String),
}

/// Build systems supported by polybuild
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildSystem {
    Cargo,
    GoMod,
    Pip,
    Npm,
    Yarn,
    Maven,
    Gradle,
    CMake,
    Make,
    Nlink,     // OBINexus nlink
    Polybuild, // OBINexus polybuild
    Other(String),
}

/// Dependency specification
#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: String,
    pub constraint: VersionConstraint,
    pub optional: bool,
    pub features: Vec<String>,
}

impl Dependency {
    /// Create a new dependency
    pub fn new(name: String, constraint: VersionConstraint) -> Self {
        Self {
            name,
            constraint,
            optional: false,
            features: Vec::new(),
        }
    }
    
    /// Mark dependency as optional
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
    
    /// Add required features
    pub fn with_features(mut self, features: Vec<String>) -> Self {
        self.features = features;
        self
    }
}

/// Trait for hot-swappable components
pub trait HotSwappable {
    /// Perform hot-swap validation
    fn validate_swap(&self, replacement: &Self) -> Result<(), String>;
    
    /// Get component identifier
    fn component_id(&self) -> String;
    
    /// Get component version
    fn component_version(&self) -> &SemverX;
}

impl HotSwappable for PackageMetadata {
    fn validate_swap(&self, replacement: &Self) -> Result<(), String> {
        if self.name != replacement.name {
            return Err(format!(
                "Package name mismatch: {} != {}",
                self.name, replacement.name
            ));
        }
        
        if self.language != replacement.language {
            return Err(format!(
                "Language mismatch: {:?} != {:?}",
                self.language, replacement.language
            ));
        }
        
        // Allow version upgrades but check compatibility
        if replacement.version.major < self.version.major {
            return Err("Cannot downgrade major version".to_string());
        }
        
        Ok(())
    }
    
    fn component_id(&self) -> String {
        self.name.clone()
    }
    
    fn component_version(&self) -> &SemverX {
        &self.version
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_parsing() {
        let v = SemverX::from_str("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        
        let v = SemverX::from_str("2.0.0-alpha").unwrap();
        assert_eq!(v.prerelease, Some("alpha".to_string()));
        
        let v = SemverX::from_str("3.1.4+build.123").unwrap();
        assert_eq!(v.build_metadata, Some("build.123".to_string()));
    }
    
    #[test]
    fn test_version_constraints() {
        let v1 = SemverX::new(1, 2, 3);
        let v2 = SemverX::new(1, 3, 0);
        let v3 = SemverX::new(2, 0, 0);
        
        let constraint = VersionConstraint::Compatible(v1.clone());
        assert!(v1.satisfies(&constraint));
        assert!(v2.satisfies(&constraint));
        assert!(!v3.satisfies(&constraint));
    }
    
    #[test]
    fn test_hot_swap_validation() {
        let pkg1 = PackageMetadata {
            name: "test-pkg".to_string(),
            version: SemverX::new(1, 0, 0),
            language: Language::Rust,
            build_system: BuildSystem::Cargo,
            dependencies: vec![],
        };
        
        let pkg2 = PackageMetadata {
            name: "test-pkg".to_string(),
            version: SemverX::new(1, 1, 0),
            language: Language::Rust,
            build_system: BuildSystem::Cargo,
            dependencies: vec![],
        };
        
        assert!(pkg1.validate_swap(&pkg2).is_ok());
        
        let pkg3 = PackageMetadata {
            name: "different-pkg".to_string(),
            version: SemverX::new(1, 1, 0),
            language: Language::Rust,
            build_system: BuildSystem::Cargo,
            dependencies: vec![],
        };
        
        assert!(pkg1.validate_swap(&pkg3).is_err());
    }
}
