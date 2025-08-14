// SEMVERX - Polyglot Semantic Versioning Extension for OBINexus
// Core implementation with error bubbling and VerbNoun taxonomy

use std::fmt;
use std::error::Error;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

// ==================== Error Model with Bubbling ====================

/// VerbNoun Error Taxonomy - Actions define errors
#[derive(Debug, Clone, PartialEq)]
pub enum VerbNounError {
    ParsingError(String),        // Error while parsing version
    ValidatingError(String),      // Error while validating
    ComparingError(String),       // Error while comparing
    ResolvingError(String),       // Error while resolving dependencies
    HealingError(String),         // Error during self-healing
    PanicError(String),          // Critical panic state
}

/// Observable error that bubbles up through the observer pattern
#[derive(Debug)]
pub struct BubblingError {
    pub source: VerbNounError,
    pub context: Vec<String>,
    pub stress_level: f64,
    pub can_recover: bool,
}

impl BubblingError {
    pub fn bubble_up(&mut self, context: &str) {
        self.context.push(context.to_string());
        self.stress_level *= 1.5; // Increase stress as error bubbles
    }
}

impl fmt::Display for BubblingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BubblingError[stress={:.2}]: {:?} | Context: {:?}", 
               self.stress_level, self.source, self.context)
    }
}

impl Error for BubblingError {}

// ==================== VerbNoun Component Taxonomy ====================

/// VerbNoun classification for components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerbNounClass {
    // Speed-related verbs
    SpeedingComponent,
    SlowingComponent,
    StoppingComponent,
    AcceleratingComponent,
    
    // State-related verbs
    LoadingComponent,
    ProcessingComponent,
    CachingComponent,
    ValidatingComponent,
    
    // Health-related verbs
    HealingComponent,
    FailingComponent,
    RecoveringComponent,
    MonitoringComponent,
}

/// Component with VerbNoun classification
#[derive(Debug, Clone)]
pub struct VerbNounComponent {
    pub class: VerbNounClass,
    pub version: SemverX,
    pub health: ComponentHealth,
    pub observers: Vec<Box<dyn ErrorObserver>>,
}

// ==================== Extended Semantic Version ====================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemverX {
    // Core semver fields
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    
    // Extended fields
    pub prerelease: Option<String>,
    pub build: Option<String>,
    pub environment: Option<Environment>,
    pub classifier: Option<Classifier>,
    pub intent: Option<String>, // 32-bit intent hash
    
    // SEI layers
    pub sei: SEIMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SEIMetadata {
    // Statement: What the component declares
    pub statement_contract: String,
    pub statement_version: u32,
    
    // Expression: How it behaves
    pub expression_signature: String,
    pub expression_complexity: u32,
    
    // Intent: Why it exists
    pub intent_hash: String,
    pub intent_priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Environment {
    Dev,
    Test,
    Staging,
    Prod,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Classifier {
    Stable,
    Legacy,
    Experimental,
    Deprecated,
}

// ==================== Stress Zone Integration ====================

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StressZone {
    Ok,        // 0-3: Normal
    Warning,   // 3-6: Enhanced monitoring
    Danger,    // 6-9: Restricted operations
    Critical,  // 9-12: Termination
}

#[derive(Debug, Clone)]
pub struct ComponentHealth {
    pub stress_level: f64,
    pub zone: StressZone,
    pub heal_attempts: u32,
    pub last_heal_timestamp: u64,
}

impl ComponentHealth {
    pub fn assess_zone(&mut self) {
        self.zone = match self.stress_level {
            s if s < 3.0 => StressZone::Ok,
            s if s < 6.0 => StressZone::Warning,
            s if s < 9.0 => StressZone::Danger,
            _ => StressZone::Critical,
        };
    }
    
    pub fn can_self_heal(&self) -> bool {
        self.stress_level < 9.0 && self.heal_attempts < 3
    }
}

// ==================== Observer Pattern for Error Bubbling ====================

pub trait ErrorObserver: Send + Sync {
    fn on_error(&mut self, error: &mut BubblingError) -> Result<(), BubblingError>;
    fn on_recovery(&mut self, component: &VerbNounComponent);
}

pub struct ErrorBubbler {
    observers: Vec<Box<dyn ErrorObserver>>,
}

impl ErrorBubbler {
    pub fn new() -> Self {
        Self { observers: Vec::new() }
    }
    
    pub fn attach(&mut self, observer: Box<dyn ErrorObserver>) {
        self.observers.push(observer);
    }
    
    pub fn bubble_error(&mut self, mut error: BubblingError) -> Result<(), BubblingError> {
        for observer in &mut self.observers {
            match observer.on_error(&mut error) {
                Ok(()) => return Ok(()), // Error handled
                Err(e) => error = e,     // Continue bubbling
            }
        }
        Err(error)
    }
}

// ==================== Version Parsing and Validation ====================

impl SemverX {
    pub fn parse(version_str: &str) -> Result<Self, BubblingError> {
        // Parse extended format: MAJOR.MINOR.PATCH-PRERELEASE+BUILD.ENV.CLASS.INTENT
        let parts: Vec<&str> = version_str.split('.').collect();
        
        if parts.len() < 3 {
            return Err(BubblingError {
                source: VerbNounError::ParsingError(
                    format!("Invalid version format: {}", version_str)
                ),
                context: vec!["SemverX::parse".to_string()],
                stress_level: 3.0,
                can_recover: false,
            });
        }
        
        // Parse core version numbers
        let major = parts[0].parse::<u32>().map_err(|e| BubblingError {
            source: VerbNounError::ParsingError(format!("Invalid major version: {}", e)),
            context: vec!["SemverX::parse::major".to_string()],
            stress_level: 3.0,
            can_recover: false,
        })?;
        
        let minor = parts[1].parse::<u32>().map_err(|e| BubblingError {
            source: VerbNounError::ParsingError(format!("Invalid minor version: {}", e)),
            context: vec!["SemverX::parse::minor".to_string()],
            stress_level: 3.0,
            can_recover: false,
        })?;
        
        // Handle patch with possible prerelease/build metadata
        let patch_part = parts[2];
        let (patch_str, prerelease, build) = Self::parse_patch_metadata(patch_part)?;
        
        let patch = patch_str.parse::<u32>().map_err(|e| BubblingError {
            source: VerbNounError::ParsingError(format!("Invalid patch version: {}", e)),
            context: vec!["SemverX::parse::patch".to_string()],
            stress_level: 3.0,
            can_recover: false,
        })?;
        
        // Parse extended fields if present
        let environment = if parts.len() > 3 {
            Some(Self::parse_environment(parts[3])?)
        } else {
            None
        };
        
        let classifier = if parts.len() > 4 {
            Some(Self::parse_classifier(parts[4])?)
        } else {
            None
        };
        
        let intent = if parts.len() > 5 {
            Some(parts[5].to_string())
        } else {
            None
        };
        
        Ok(SemverX {
            major,
            minor,
            patch,
            prerelease,
            build,
            environment,
            classifier,
            intent,
            sei: SEIMetadata::default(),
        })
    }
    
    fn parse_patch_metadata(patch_part: &str) -> Result<(String, Option<String>, Option<String>), BubblingError> {
        // Handle -prerelease and +build metadata
        let mut patch = patch_part.to_string();
        let mut prerelease = None;
        let mut build = None;
        
        if let Some(idx) = patch.find('-') {
            let (p, pre) = patch.split_at(idx);
            patch = p.to_string();
            let pre = &pre[1..]; // Skip the '-'
            
            if let Some(build_idx) = pre.find('+') {
                let (pre_part, build_part) = pre.split_at(build_idx);
                prerelease = Some(pre_part.to_string());
                build = Some(build_part[1..].to_string()); // Skip the '+'
            } else {
                prerelease = Some(pre.to_string());
            }
        } else if let Some(idx) = patch.find('+') {
            let (p, b) = patch.split_at(idx);
            patch = p.to_string();
            build = Some(b[1..].to_string()); // Skip the '+'
        }
        
        Ok((patch, prerelease, build))
    }
    
    fn parse_environment(env_str: &str) -> Result<Environment, BubblingError> {
        match env_str.to_lowercase().as_str() {
            "dev" => Ok(Environment::Dev),
            "test" => Ok(Environment::Test),
            "staging" => Ok(Environment::Staging),
            "prod" => Ok(Environment::Prod),
            _ => Err(BubblingError {
                source: VerbNounError::ParsingError(format!("Unknown environment: {}", env_str)),
                context: vec!["SemverX::parse_environment".to_string()],
                stress_level: 2.0,
                can_recover: true,
            })
        }
    }
    
    fn parse_classifier(class_str: &str) -> Result<Classifier, BubblingError> {
        match class_str.to_lowercase().as_str() {
            "stable" => Ok(Classifier::Stable),
            "legacy" => Ok(Classifier::Legacy),
            "experimental" => Ok(Classifier::Experimental),
            "deprecated" => Ok(Classifier::Deprecated),
            _ => Err(BubblingError {
                source: VerbNounError::ParsingError(format!("Unknown classifier: {}", class_str)),
                context: vec!["SemverX::parse_classifier".to_string()],
                stress_level: 2.0,
                can_recover: true,
            })
        }
    }
    
    /// Validate version according to OBINexus policies
    pub fn validate(&self) -> Result<(), BubblingError> {
        // Validate SEI metadata
        if self.sei.intent_priority > 100 {
            return Err(BubblingError {
                source: VerbNounError::ValidatingError(
                    "Intent priority must be between 0-100".to_string()
                ),
                context: vec!["SemverX::validate::sei".to_string()],
                stress_level: 4.0,
                can_recover: false,
            });
        }
        
        // Validate version consistency
        if self.classifier == Some(Classifier::Deprecated) && self.environment == Some(Environment::Prod) {
            return Err(BubblingError {
                source: VerbNounError::ValidatingError(
                    "Deprecated components cannot be deployed to production".to_string()
                ),
                context: vec!["SemverX::validate::deployment".to_string()],
                stress_level: 7.0,
                can_recover: false,
            });
        }
        
        Ok(())
    }
}

impl Default for SEIMetadata {
    fn default() -> Self {
        Self {
            statement_contract: String::new(),
            statement_version: 0,
            expression_signature: String::new(),
            expression_complexity: 0,
            intent_hash: String::new(),
            intent_priority: 50,
        }
    }
}

// ==================== Version Comparison ====================

impl SemverX {
    pub fn satisfies(&self, range: &str) -> Result<bool, BubblingError> {
        // Implement semver range satisfaction with stress-aware comparison
        let range_parts: Vec<&str> = range.split("||").collect();
        
        for part in range_parts {
            if self.satisfies_comparator_set(part.trim())? {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    fn satisfies_comparator_set(&self, comparator_set: &str) -> Result<bool, BubblingError> {
        // Parse and evaluate comparator set (e.g., ">=1.2.3 <2.0.0")
        let comparators: Vec<&str> = comparator_set.split_whitespace().collect();
        
        for comparator in comparators {
            if !self.satisfies_single_comparator(comparator)? {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    fn satisfies_single_comparator(&self, comparator: &str) -> Result<bool, BubblingError> {
        // Parse operator and version
        let (op, version_str) = if comparator.starts_with(">=") {
            (">=", &comparator[2..])
        } else if comparator.starts_with("<=") {
            ("<=", &comparator[2..])
        } else if comparator.starts_with('>') {
            (">", &comparator[1..])
        } else if comparator.starts_with('<') {
            ("<", &comparator[1..])
        } else if comparator.starts_with('=') {
            ("=", &comparator[1..])
        } else {
            ("=", comparator)
        };
        
        let other = SemverX::parse(version_str)?;
        
        Ok(match op {
            ">=" => self.compare(&other) >= 0,
            "<=" => self.compare(&other) <= 0,
            ">" => self.compare(&other) > 0,
            "<" => self.compare(&other) < 0,
            "=" => self.compare(&other) == 0,
            _ => false,
        })
    }
    
    pub fn compare(&self, other: &SemverX) -> i32 {
        // Compare versions with precedence rules
        if self.major != other.major {
            return self.major.cmp(&other.major) as i32;
        }
        if self.minor != other.minor {
            return self.minor.cmp(&other.minor) as i32;
        }
        if self.patch != other.patch {
            return self.patch.cmp(&other.patch) as i32;
        }
        
        // Handle prerelease comparison
        match (&self.prerelease, &other.prerelease) {
            (None, None) => 0,
            (None, Some(_)) => 1,  // No prerelease > prerelease
            (Some(_), None) => -1,
            (Some(a), Some(b)) => a.cmp(b) as i32,
        }
    }
}

// ==================== Self-Healing Mechanism ====================

impl VerbNounComponent {
    pub async fn attempt_self_heal(&mut self) -> Result<(), BubblingError> {
        if !self.health.can_self_heal() {
            return Err(BubblingError {
                source: VerbNounError::HealingError(
                    "Component cannot self-heal in current state".to_string()
                ),
                context: vec!["VerbNounComponent::attempt_self_heal".to_string()],
                stress_level: self.health.stress_level,
                can_recover: false,
            });
        }
        
        self.health.heal_attempts += 1;
        
        // Healing strategy based on VerbNoun class
        match self.class {
            VerbNounClass::FailingComponent => {
                // Attempt recovery through rollback
                self.rollback_to_stable()?;
            },
            VerbNounClass::SlowingComponent => {
                // Clear caches and restart
                self.clear_and_restart()?;
            },
            VerbNounClass::StoppingComponent => {
                // Force restart with new configuration
                self.force_restart()?;
            },
            _ => {
                // Generic healing
                self.generic_heal()?;
            }
        }
        
        // Reassess health after healing
        self.health.stress_level *= 0.5; // Reduce stress by half
        self.health.assess_zone();
        
        Ok(())
    }
    
    fn rollback_to_stable(&mut self) -> Result<(), BubblingError> {
        // Implementation for rollback
        Ok(())
    }
    
    fn clear_and_restart(&mut self) -> Result<(), BubblingError> {
        // Implementation for cache clearing
        Ok(())
    }
    
    fn force_restart(&mut self) -> Result<(), BubblingError> {
        // Implementation for forced restart
        Ok(())
    }
    
    fn generic_heal(&mut self) -> Result<(), BubblingError> {
        // Generic healing implementation
        Ok(())
    }
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_basic_version() {
        let version = SemverX::parse("1.2.3").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }
    
    #[test]
    fn test_parse_with_prerelease() {
        let version = SemverX::parse("1.2.3-alpha.1").unwrap();
        assert_eq!(version.prerelease, Some("alpha.1".to_string()));
    }
    
    #[test]
    fn test_parse_with_build() {
        let version = SemverX::parse("1.2.3+build.123").unwrap();
        assert_eq!(version.build, Some("build.123".to_string()));
    }
    
    #[test]
    fn test_stress_zone_assessment() {
        let mut health = ComponentHealth {
            stress_level: 7.5,
            zone: StressZone::Ok,
            heal_attempts: 0,
            last_heal_timestamp: 0,
        };
        
        health.assess_zone();
        assert_eq!(health.zone, StressZone::Danger);
    }
    
    #[test]
    fn test_error_bubbling() {
        let mut error = BubblingError {
            source: VerbNounError::ParsingError("test".to_string()),
            context: vec![],
            stress_level: 1.0,
            can_recover: true,
        };
        
        error.bubble_up("context1");
        error.bubble_up("context2");
        
        assert_eq!(error.context.len(), 2);
        assert!(error.stress_level > 2.0);
    }
}