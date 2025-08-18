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
