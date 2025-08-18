use std::fmt;
use std::error::Error;
use std::cmp::Ordering;
use unicode_normalization::UnicodeNormalization;

pub trait ErrorObserver: fmt::Debug + Send + Sync {
    fn observe(&self, error: &dyn Error);
}

#[derive(Debug, Clone)]
pub struct DefaultErrorObserver;

impl ErrorObserver for DefaultErrorObserver {
    fn observe(&self, error: &dyn Error) {
        eprintln!("[ERROR] {}", error);
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Version {
    pub major: u64,
    pub minor: u64,
    pub patch: u64,
    pub pre: Option<String>,
    pub build: Option<String>,
}

impl Version {
    pub fn new(major: u64, minor: u64, patch: u64) -> Self {
        Version {
            major,
            minor,
            patch,
            pre: None,
            build: None,
        }
    }

    pub fn parse(version: &str) -> Result<Self, String> {
        let version = version.trim();
        if version.is_empty() {
            return Err("Empty version string".to_string());
        }

        let mut parts = version.split('.');
        
        let major = parts.next()
            .ok_or("Missing major version")?
            .parse::<u64>()
            .map_err(|e| format!("Invalid major version: {}", e))?;
            
        let minor = parts.next()
            .ok_or("Missing minor version")?
            .parse::<u64>()
            .map_err(|e| format!("Invalid minor version: {}", e))?;
            
        let patch_part = parts.next()
            .ok_or("Missing patch version")?;
            
        let (patch, pre, build) = Self::parse_patch_part(patch_part)?;
        
        Ok(Version {
            major,
            minor,
            patch,
            pre,
            build,
        })
    }
    
    fn parse_patch_part(patch_part: &str) -> Result<(u64, Option<String>, Option<String>), String> {
        let patch_str = patch_part.to_string();
        let pre;
        let build;
        
        // Check for prerelease
        if let Some(idx) = patch_str.find('-') {
            let (p, rest) = patch_str.split_at(idx);
            let patch = p.parse::<u64>()
                .map_err(|e| format!("Invalid patch version: {}", e))?;
            
            // Check for build metadata
            if let Some(build_idx) = rest.find('+') {
                let (pre_part, build_part) = rest.split_at(build_idx);
                pre = Some(pre_part[1..].to_string());
                build = Some(build_part[1..].to_string());
            } else {
                pre = Some(rest[1..].to_string());
                build = None;
            }
            
            Ok((patch, pre, build))
        } else if let Some(idx) = patch_str.find('+') {
            let (p, b) = patch_str.split_at(idx);
            let patch = p.parse::<u64>()
                .map_err(|e| format!("Invalid patch version: {}", e))?;
            build = Some(b[1..].to_string());
            Ok((patch, None, build))
        } else {
            let patch = patch_str.parse::<u64>()
                .map_err(|e| format!("Invalid patch version: {}", e))?;
            Ok((patch, None, None))
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(ref pre) = self.pre {
            write!(f, "-{}", pre)?;
        }
        if let Some(ref build) = self.build {
            write!(f, "+{}", build)?;
        }
        Ok(())
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => {},
            ord => return ord,
        }
        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {},
            ord => return ord,
        }
        match self.patch.cmp(&other.patch) {
            Ordering::Equal => {},
            ord => return ord,
        }
        
        // Handle prerelease comparison
        match (&self.pre, &other.pre) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(a), Some(b)) => a.cmp(b),
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
pub struct OBINexusSemverX {
    pub version: Version,
    pub security_mode: SecurityMode,
    pub observers: Vec<Box<DefaultErrorObserver>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityMode {
    Standard,
    ZeroTrust,
    Hardened,
}

impl OBINexusSemverX {
    pub fn new(version: Version) -> Self {
        OBINexusSemverX {
            version,
            security_mode: SecurityMode::Standard,
            observers: vec![Box::new(DefaultErrorObserver)],
        }
    }
    
    pub fn with_security(mut self, mode: SecurityMode) -> Self {
        self.security_mode = mode;
        self
    }
    
    pub fn normalize_unicode_path(&self, path: &str) -> String {
        path.nfc().collect::<String>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_parsing() {
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        assert_eq!(v.pre, None);
        assert_eq!(v.build, None);
    }
    
    #[test]
    fn test_version_with_prerelease() {
        let v = Version::parse("1.0.0-alpha").unwrap();
        assert_eq!(v.pre, Some("alpha".to_string()));
    }
    
    #[test]
    fn test_version_with_build() {
        let v = Version::parse("1.0.0+build123").unwrap();
        assert_eq!(v.build, Some("build123".to_string()));
    }
}
