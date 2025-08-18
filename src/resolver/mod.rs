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
