//! Dependency resolution using Eulerian/Hamiltonian cycle algorithms
//! Part of OBINexus polyglot package management

use std::collections::{HashMap, HashSet, VecDeque};
use std::error::Error;
use std::fmt;

/// Custom error type for dependency resolution
#[derive(Debug)]
pub enum ResolverError {
    CircularDependency(String),
    MissingDependency(String),
    VersionConflict(String),
}

impl fmt::Display for ResolverError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ResolverError::CircularDependency(msg) => write!(f, "Circular dependency: {}", msg),
            ResolverError::MissingDependency(msg) => write!(f, "Missing dependency: {}", msg),
            ResolverError::VersionConflict(msg) => write!(f, "Version conflict: {}", msg),
        }
    }
}

impl Error for ResolverError {}

/// Represents a package node in the dependency graph
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PackageNode {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<String>,
}

/// Dependency resolver using graph algorithms
pub struct DependencyResolver {
    graph: HashMap<String, PackageNode>,
    visited: HashSet<String>,
    resolution_order: Vec<String>,
}

impl DependencyResolver {
    /// Create a new dependency resolver
    pub fn new() -> Self {
        Self {
            graph: HashMap::new(),
            visited: HashSet::new(),
            resolution_order: Vec::new(),
        }
    }

    /// Add a package to the dependency graph
    pub fn add_package(&mut self, package: PackageNode) {
        self.graph.insert(package.name.clone(), package);
    }

    /// Detect Eulerian cycles in the dependency graph
    pub fn has_eulerian_cycle(&self) -> bool {
        // Check if all vertices have even degree (simplified for package deps)
        for (_, node) in &self.graph {
            if node.dependencies.len() % 2 != 0 {
                return false;
            }
        }
        true
    }

    /// Detect Hamiltonian cycles using DFS
    pub fn has_hamiltonian_cycle(&self) -> bool {
        if self.graph.is_empty() {
            return false;
        }

        let start = self.graph.keys().next().unwrap().clone();
        let mut path = vec![start.clone()];
        let mut visited = HashSet::new();
        visited.insert(start.clone());

        self.hamiltonian_dfs(&start, &start, &mut visited, &mut path)
    }

    fn hamiltonian_dfs(
        &self,
        current: &str,
        start: &str,
        visited: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> bool {
        if path.len() == self.graph.len() {
            // Check if we can return to start
            if let Some(node) = self.graph.get(current) {
                return node.dependencies.contains(&start.to_string());
            }
        }

        if let Some(node) = self.graph.get(current) {
            for dep in &node.dependencies {
                if !visited.contains(dep) {
                    visited.insert(dep.clone());
                    path.push(dep.clone());

                    if self.hamiltonian_dfs(dep, start, visited, path) {
                        return true;
                    }

                    path.pop();
                    visited.remove(dep);
                }
            }
        }

        false
    }

    /// Resolve dependencies using topological sort
    pub fn resolve(&mut self) -> Result<Vec<String>, ResolverError> {
        // Detect circular dependencies first
        if self.detect_circular_dependencies()? {
            return Err(ResolverError::CircularDependency(
                "Circular dependency detected in package graph".to_string()
            ));
        }

        // Perform topological sort
        self.visited.clear();
        self.resolution_order.clear();

        for package_name in self.graph.keys().cloned().collect::<Vec<_>>() {
            if !self.visited.contains(&package_name) {
                self.dfs_visit(&package_name)?;
            }
        }

        self.resolution_order.reverse();
        Ok(self.resolution_order.clone())
    }

    fn dfs_visit(&mut self, package_name: &str) -> Result<(), ResolverError> {
        self.visited.insert(package_name.to_string());

        if let Some(node) = self.graph.get(package_name).cloned() {
            for dep in &node.dependencies {
                if !self.visited.contains(dep) {
                    if !self.graph.contains_key(dep) {
                        return Err(ResolverError::MissingDependency(dep.clone()));
                    }
                    self.dfs_visit(dep)?;
                }
            }
        }

        self.resolution_order.push(package_name.to_string());
        Ok(())
    }

    fn detect_circular_dependencies(&self) -> Result<bool, ResolverError> {
        let mut white = HashSet::new();
        let mut gray = HashSet::new();
        let mut black = HashSet::new();

        for key in self.graph.keys() {
            white.insert(key.clone());
        }

        while !white.is_empty() {
            let node = white.iter().next().cloned().unwrap();
            if self.has_cycle(&node, &mut white, &mut gray, &mut black)? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    fn has_cycle(
        &self,
        node: &str,
        white: &mut HashSet<String>,
        gray: &mut HashSet<String>,
        black: &mut HashSet<String>,
    ) -> Result<bool, ResolverError> {
        white.remove(node);
        gray.insert(node.to_string());

        if let Some(package) = self.graph.get(node) {
            for dep in &package.dependencies {
                if gray.contains(dep) {
                    return Ok(true); // Cycle detected
                }
                if white.contains(dep) {
                    if self.has_cycle(dep, white, gray, black)? {
                        return Ok(true);
                    }
                }
            }
        }

        gray.remove(node);
        black.insert(node.to_string());
        Ok(false)
    }

    /// Hot-swap a package component
    pub fn hot_swap(&mut self, old_package: &str, new_package: PackageNode) -> Result<(), ResolverError> {
        if !self.graph.contains_key(old_package) {
            return Err(ResolverError::MissingDependency(
                format!("Package {} not found for hot-swap", old_package)
            ));
        }

        // Update references to old package
        for (_, node) in self.graph.iter_mut() {
            for dep in &mut node.dependencies {
                if dep == old_package {
                    *dep = new_package.name.clone();
                }
            }
        }

        // Remove old and insert new
        self.graph.remove(old_package);
        self.graph.insert(new_package.name.clone(), new_package);

        Ok(())
    }
}

impl Default for DependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_resolution() {
        let mut resolver = DependencyResolver::new();
        
        resolver.add_package(PackageNode {
            name: "a".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec!["b".to_string()],
        });
        
        resolver.add_package(PackageNode {
            name: "b".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec![],
        });

        let order = resolver.resolve().unwrap();
        assert_eq!(order, vec!["b", "a"]);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut resolver = DependencyResolver::new();
        
        resolver.add_package(PackageNode {
            name: "a".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec!["b".to_string()],
        });
        
        resolver.add_package(PackageNode {
            name: "b".to_string(),
            version: "1.0.0".to_string(),
            dependencies: vec!["a".to_string()],
        });

        assert!(resolver.resolve().is_err());
    }
}
