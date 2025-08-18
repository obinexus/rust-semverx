// resolver/types.rs - Core type definitions

use crate::{SemverX, ComponentHealth};

#[derive(Debug, Clone)]
pub struct Component {
    pub id: String,
    pub version: SemverX,
    pub dependencies: Vec<Dependency>,
    pub health: ComponentHealth,
    pub resolution_attempts: u32,
}

#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: String,
    pub version: SemverX,
    pub range: String,
    pub optional: bool,
    pub dev: bool,
    pub verb_noun_class: String,
}

#[derive(Debug, Clone)]
pub struct ComponentNode {
    pub id: String,
    pub version: SemverX,
    pub dependencies: Vec<Dependency>,
    pub health: ComponentHealth,
    pub resolution_attempts: u32,
}

#[derive(Debug, Clone)]
pub struct DependencyEdge {
    pub constraint: String,
    pub weight: f64, // Cost/stress of this dependency
    pub is_critical: bool,
}

impl Component {
    pub fn new(id: String, version: SemverX) -> Self {
        Self {
            id,
            version,
            dependencies: Vec::new(),
            health: ComponentHealth::default(),
            resolution_attempts: 0,
        }
    }
    
    pub fn add_dependency(&mut self, dep: Dependency) {
        self.dependencies.push(dep);
    }
}

impl Dependency {
    pub fn new(name: String, version: SemverX, range: String) -> Self {
        Self {
            name,
            version,
            range,
            optional: false,
            dev: false,
            verb_noun_class: String::new(),
        }
    }
    
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
    
    pub fn dev(mut self) -> Self {
        self.dev = true;
        self
    }
}
