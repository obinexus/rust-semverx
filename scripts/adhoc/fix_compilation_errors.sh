#!/bin/bash
echo "Fixing rust-semverx compilation errors..."

# Fix ErrorObserver trait bounds
cat > src/core/semver.rs << 'EOL'
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
EOL

# Fix resolver graph issues
cat > src/resolver/graph.rs << 'EOL'
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::dijkstra;
use std::collections::HashMap;
use std::fmt;

use super::{DependencyResolver, ResolutionError, Component};

pub struct GraphResolver {
    graph: DiGraph<Component, f64>,
    node_map: HashMap<String, NodeIndex>,
}

impl GraphResolver {
    pub fn new() -> Self {
        GraphResolver {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
        }
    }
    
    pub fn add_component(&mut self, component: Component) -> NodeIndex {
        let name = component.name.clone();
        let idx = self.graph.add_node(component);
        self.node_map.insert(name, idx);
        idx
    }
    
    pub fn add_dependency(&mut self, from: &str, to: &str, weight: f64) {
        if let (Some(&from_idx), Some(&to_idx)) = 
            (self.node_map.get(from), self.node_map.get(to)) {
            self.graph.add_edge(from_idx, to_idx, weight);
        }
    }
    
    pub fn resolve_path(&self, from: &str, to: &str) -> Result<Vec<String>, ResolutionError> {
        let from_idx = self.node_map.get(from)
            .ok_or_else(|| ResolutionError::ComponentNotFound(from.to_string()))?;
        let to_idx = self.node_map.get(to)
            .ok_or_else(|| ResolutionError::ComponentNotFound(to.to_string()))?;
            
        let paths = dijkstra(&self.graph, *from_idx, Some(*to_idx), |_| 1.0);
        
        if paths.contains_key(to_idx) {
            // Build path by backtracking
            let mut path = vec![to.to_string()];
            let mut current = *to_idx;
            
            // Simple path reconstruction (would need proper predecessor tracking)
            while current != *from_idx {
                // Find predecessor
                for edge in self.graph.edges_directed(current, petgraph::Direction::Incoming) {
                    let predecessor = edge.source();
                    if paths.contains_key(&predecessor) {
                        if let Some(node) = self.graph.node_weight(predecessor) {
                            path.push(node.name.clone());
                            current = predecessor;
                            break;
                        }
                    }
                }
            }
            
            path.reverse();
            Ok(path)
        } else {
            Err(ResolutionError::NoPathFound(from.to_string(), to.to_string()))
        }
    }
}

#[derive(Debug)]
pub struct SemverXResolver {
    pub graph: GraphResolver,
    cache: HashMap<String, Vec<String>>,
}

impl SemverXResolver {
    pub fn new() -> Self {
        SemverXResolver {
            graph: GraphResolver::new(),
            cache: HashMap::new(),
        }
    }
    
    pub fn add_package(&mut self, component: Component) {
        self.graph.add_component(component);
    }
    
    pub fn add_dependency(&mut self, from: &str, to: &str) {
        self.graph.add_dependency(from, to, 1.0);
    }
    
    pub fn resolve(&mut self, package: &str, version: &str) -> Result<Vec<String>, ResolutionError> {
        let key = format!("{}@{}", package, version);
        
        if let Some(cached) = self.cache.get(&key) {
            return Ok(cached.clone());
        }
        
        // Simple resolution - just return the package for now
        let result = vec![key.clone()];
        self.cache.insert(key, result.clone());
        Ok(result)
    }
    
    fn extract_versions(&self, path: &[String]) -> Vec<String> {
        path.to_vec()
    }
}

impl DependencyResolver for SemverXResolver {
    fn resolve_dependencies(&mut self, package: &str, version: &str) 
        -> Result<Vec<Component>, ResolutionError> {
        let resolved_paths = self.resolve(package, version)?;
        
        Ok(resolved_paths.into_iter()
            .map(|path| Component {
                name: path.clone(),
                version: "1.0.0".to_string(),
                dependencies: vec![],
            })
            .collect())
    }
    
    fn add_constraint(&mut self, _package: &str, _constraint: &str) {
        // Implement constraint handling
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_graph_resolver() {
        let mut resolver = GraphResolver::new();
        
        let comp_a = Component {
            name: "a".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec![],
        };
        
        let comp_b = Component {
            name: "b".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec![],
        };
        
        resolver.add_component(comp_a);
        resolver.add_component(comp_b);
        resolver.add_dependency("a", "b", 1.0);
        
        let path = resolver.resolve_path("a", "b");
        assert!(path.is_ok());
    }
}
EOL

# Update resolver mod.rs
cat > src/resolver/mod.rs << 'EOL'
pub mod graph;

use std::fmt;
use std::error::Error;

pub use graph::{GraphResolver, SemverXResolver};

#[derive(Debug, Clone)]
pub struct Component {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: String,
    pub version_req: String,
}

#[derive(Debug)]
pub enum ResolutionError {
    ComponentNotFound(String),
    VersionConflict(String, String),
    NoPathFound(String, String),
    CyclicDependency(Vec<String>),
}

impl fmt::Display for ResolutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolutionError::ComponentNotFound(name) => 
                write!(f, "Component not found: {}", name),
            ResolutionError::VersionConflict(pkg, ver) => 
                write!(f, "Version conflict: {} @ {}", pkg, ver),
            ResolutionError::NoPathFound(from, to) => 
                write!(f, "No path found from {} to {}", from, to),
            ResolutionError::CyclicDependency(cycle) => 
                write!(f, "Cyclic dependency detected: {:?}", cycle),
        }
    }
}

impl Error for ResolutionError {}

pub trait DependencyResolver {
    fn resolve_dependencies(&mut self, package: &str, version: &str) 
        -> Result<Vec<Component>, ResolutionError>;
    fn add_constraint(&mut self, package: &str, constraint: &str);
}
EOL

# Add normalizer implementation
cat > src/normalizer/mod.rs << 'EOL'
use unicode_normalization::UnicodeNormalization;

pub struct UnicodeNormalizer;

impl UnicodeNormalizer {
    pub fn new() -> Self {
        UnicodeNormalizer
    }
    
    pub fn normalize(&self, text: &str) -> String {
        text.nfc().collect()
    }
}

pub fn normalize_unicode_path(path: &str) -> String {
    path.nfc().collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_normalization() {
        let normalizer = UnicodeNormalizer::new();
        let input = "café";
        let normalized = normalizer.normalize(input);
        assert!(!normalized.is_empty());
    }
}
EOL

echo "Compilation fixes applied!"
