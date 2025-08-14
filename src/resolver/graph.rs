// SEMVERX Dependency Resolver with Graph-Based Resolution
// Implements Eulerian, Hamiltonian, A* and Hybrid strategies

use std::collections::{HashMap, HashSet, VecDeque, BinaryHeap};
use std::cmp::Ordering;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::{tarjan_scc, has_path_connecting};
use petgraph::visit::EdgeRef;
use crate::{SemverX, BubblingError, VerbNounError, StressZone, ComponentHealth};

// ==================== Dependency Graph Structure ====================

#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: String,
    pub version: SemverX,
    pub range: String,
    pub optional: bool,
    pub dev: bool,
    pub verb_noun_class: String,
}

#[derive(Debug)]
pub struct DependencyGraph {
    graph: DiGraph<ComponentNode, DependencyEdge>,
    node_map: HashMap<String, NodeIndex>,
    stress_monitor: StressMonitor,
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

// ==================== Resolution Strategies ====================

#[derive(Debug, Clone, PartialEq)]
pub enum ResolutionStrategy {
    Eulerian,    // Visit all edges (comprehensive)
    Hamiltonian, // Direct node path (fast)
    AStar,       // Nearest viable path (optimal)
    Hybrid,      // Adaptive based on stress
}

pub struct SemverXResolver {
    graph: DependencyGraph,
    strategy: ResolutionStrategy,
    max_iterations: usize,
    bubble_errors: bool,
}

impl SemverXResolver {
    pub fn new() -> Self {
        Self {
            graph: DependencyGraph::new(),
            strategy: ResolutionStrategy::Hybrid,
            max_iterations: 1000,
            bubble_errors: true,
        }
    }
    
    /// Main resolution entry point with adaptive strategy
    pub fn resolve(
        &mut self, 
        component_id: &str, 
        strategy: Option<ResolutionStrategy>
    ) -> Result<ResolutionResult, BubblingError> {
        // Determine strategy based on stress level if not specified
        let strategy = strategy.unwrap_or_else(|| {
            self.determine_strategy_from_stress()
        });
        
        match strategy {
            ResolutionStrategy::Eulerian => self.resolve_eulerian(component_id),
            ResolutionStrategy::Hamiltonian => self.resolve_hamiltonian(component_id),
            ResolutionStrategy::AStar => self.resolve_a_star(component_id),
            ResolutionStrategy::Hybrid => self.resolve_hybrid(component_id),
        }
    }
    
    fn determine_strategy_from_stress(&self) -> ResolutionStrategy {
        let stress = self.graph.stress_monitor.current_stress();
        
        match stress {
            s if s < 3.0 => ResolutionStrategy::Hamiltonian,
            s if s < 6.0 => ResolutionStrategy::AStar,
            s if s < 9.0 => ResolutionStrategy::Eulerian,
            _ => ResolutionStrategy::Hybrid, // Fallback to most adaptive
        }
    }
}

// ==================== Eulerian Resolution (Visit All Edges) ====================

impl SemverXResolver {
    fn resolve_eulerian(&mut self, component_id: &str) -> Result<ResolutionResult, BubblingError> {
        let start_node = self.graph.get_node(component_id)?;
        let mut visited_edges = HashSet::new();
        let mut resolution_path = Vec::new();
        let mut iterations = 0;
        
        // Build Eulerian path visiting all dependency edges
        let mut stack = vec![start_node];
        let mut current_path = Vec::new();
        
        while !stack.is_empty() && iterations < self.max_iterations {
            iterations += 1;
            
            let node = stack.last().unwrap().clone();
            let edges = self.graph.get_edges(&node);
            
            let mut found_unvisited = false;
            for edge in edges {
                let edge_id = format!("{}->{}", node, edge.target);
                if !visited_edges.contains(&edge_id) {
                    visited_edges.insert(edge_id.clone());
                    stack.push(edge.target.clone());
                    found_unvisited = true;
                    
                    // Validate version compatibility
                    self.validate_edge_compatibility(&node, &edge)?;
                    break;
                }
            }
            
            if !found_unvisited {
                if let Some(node) = stack.pop() {
                    current_path.push(node);
                }
            }
        }
        
        // Reverse to get correct order
        current_path.reverse();
        resolution_path = current_path;
        
        Ok(ResolutionResult {
            resolved_versions: self.extract_versions(&resolution_path),
            strategy_used: ResolutionStrategy::Eulerian,
            iterations,
            stress_impact: visited_edges.len() as f64 * 0.1,
        })
    }
}

// ==================== Hamiltonian Resolution (Direct Path) ====================

impl SemverXResolver {
    fn resolve_hamiltonian(&mut self, component_id: &str) -> Result<ResolutionResult, BubblingError> {
        let start_idx = self.graph.node_map.get(component_id)
            .ok_or_else(|| BubblingError {
                source: VerbNounError::ResolvingError(format!("Component not found: {}", component_id)),
                context: vec!["resolve_hamiltonian".to_string()],
                stress_level: 4.0,
                can_recover: false,
            })?;
        
        let mut visited = HashSet::new();
        let mut path = Vec::new();
        let mut iterations = 0;
        
        // Find Hamiltonian path (visit each node exactly once)
        if self.find_hamiltonian_path(*start_idx, &mut visited, &mut path, &mut iterations)? {
            Ok(ResolutionResult {
                resolved_versions: self.extract_versions_from_indices(&path),
                strategy_used: ResolutionStrategy::Hamiltonian,
                iterations,
                stress_impact: path.len() as f64 * 0.05,
            })
        } else {
            // Fallback to A* if Hamiltonian path not found
            self.resolve_a_star(component_id)
        }
    }
    
    fn find_hamiltonian_path(
        &self,
        current: NodeIndex,
        visited: &mut HashSet<NodeIndex>,
        path: &mut Vec<NodeIndex>,
        iterations: &mut usize,
    ) -> Result<bool, BubblingError> {
        *iterations += 1;
        if *iterations > self.max_iterations {
            return Ok(false);
        }
        
        visited.insert(current);
        path.push(current);
        
        // Check if we've visited all nodes
        if visited.len() == self.graph.graph.node_count() {
            return Ok(true);
        }
        
        // Try each neighbor
        let neighbors: Vec<_> = self.graph.graph
            .edges(current)
            .map(|e| e.target())
            .collect();
        
        for neighbor in neighbors {
            if !visited.contains(&neighbor) {
                if self.find_hamiltonian_path(neighbor, visited, path, iterations)? {
                    return Ok(true);
                }
            }
        }
        
        // Backtrack
        visited.remove(&current);
        path.pop();
        Ok(false)
    }
}

// ==================== A* Resolution (Optimal Path) ====================

#[derive(Clone)]
struct AStarNode {
    index: NodeIndex,
    cost: f64,
    heuristic: f64,
    parent: Option<NodeIndex>,
}

impl PartialEq for AStarNode {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl Eq for AStarNode {}

impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.total_cost().partial_cmp(&self.total_cost())
    }
}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

impl AStarNode {
    fn total_cost(&self) -> f64 {
        self.cost + self.heuristic
    }
}

impl SemverXResolver {
    fn resolve_a_star(&mut self, component_id: &str) -> Result<ResolutionResult, BubblingError> {
        let start_idx = *self.graph.node_map.get(component_id)
            .ok_or_else(|| BubblingError {
                source: VerbNounError::ResolvingError(format!("Component not found: {}", component_id)),
                context: vec!["resolve_a_star".to_string()],
                stress_level: 4.0,
                can_recover: false,
            })?;
        
        let mut open_set = BinaryHeap::new();
        let mut closed_set = HashSet::new();
        let mut came_from = HashMap::new();
        let mut g_score = HashMap::new();
        let mut iterations = 0;
        
        // Initialize with start node
        open_set.push(AStarNode {
            index: start_idx,
            cost: 0.0,
            heuristic: self.heuristic_cost(start_idx),
            parent: None,
        });
        g_score.insert(start_idx, 0.0);
        
        while let Some(current) = open_set.pop() {
            iterations += 1;
            if iterations > self.max_iterations {
                break;
            }
            
            if closed_set.contains(&current.index) {
                continue;
            }
            
            closed_set.insert(current.index);
            
            // Check if we've reached a satisfactory resolution
            if self.is_resolution_complete(&closed_set) {
                let path = self.reconstruct_path(&came_from, current.index);
                return Ok(ResolutionResult {
                    resolved_versions: self.extract_versions_from_indices(&path),
                    strategy_used: ResolutionStrategy::AStar,
                    iterations,
                    stress_impact: current.cost * 0.1,
                });
            }
            
            // Explore neighbors
            for edge in self.graph.graph.edges(current.index) {
                let neighbor = edge.target();
                let edge_weight = edge.weight().weight;
                
                let tentative_g_score = g_score[&current.index] + edge_weight;
                
                if tentative_g_score < *g_score.get(&neighbor).unwrap_or(&f64::INFINITY) {
                    came_from.insert(neighbor, current.index);
                    g_score.insert(neighbor, tentative_g_score);
                    
                    open_set.push(AStarNode {
                        index: neighbor,
                        cost: tentative_g_score,
                        heuristic: self.heuristic_cost(neighbor),
                        parent: Some(current.index),
                    });
                }
            }
        }
        
        Err(BubblingError {
            source: VerbNounError::ResolvingError("A* resolution failed to find path".to_string()),
            context: vec!["resolve_a_star".to_string()],
            stress_level: 6.0,
            can_recover: true,
        })
    }
    
    fn heuristic_cost(&self, node: NodeIndex) -> f64 {
        // Heuristic based on node health and dependency count
        if let Some(component) = self.graph.graph.node_weight(node) {
            let health_cost = component.health.stress_level;
            let dependency_cost = component.dependencies.len() as f64;
            health_cost + dependency_cost * 0.5
        } else {
            100.0 // High cost for missing nodes
        }
    }
    
    fn reconstruct_path(&self, came_from: &HashMap<NodeIndex, NodeIndex>, end: NodeIndex) -> Vec<NodeIndex> {
        let mut path = vec![end];
        let mut current = end;
        
        while let Some(&parent) = came_from.get(&current) {
            path.push(parent);
            current = parent;
        }
        
        path.reverse();
        path
    }
}

// ==================== Hybrid Resolution (Adaptive) ====================

impl SemverXResolver {
    fn resolve_hybrid(&mut self, component_id: &str) -> Result<ResolutionResult, BubblingError> {
        let stress = self.graph.stress_monitor.current_stress();
        
        // Try strategies in order of increasing complexity
        let strategies = if stress < 6.0 {
            vec![
                ResolutionStrategy::Hamiltonian,
                ResolutionStrategy::AStar,
                ResolutionStrategy::Eulerian,
            ]
        } else {
            vec![
                ResolutionStrategy::AStar,
                ResolutionStrategy::Eulerian,
                ResolutionStrategy::Hamiltonian,
            ]
        };
        
        let mut last_error = None;
        
        for strategy in strategies {
            match self.resolve(component_id, Some(strategy)) {
                Ok(result) => return Ok(result),
                Err(e) => {
                    if e.can_recover {
                        last_error = Some(e);
                        continue;
                    } else {
                        return Err(e);
                    }
                }
            }
        }
        
        Err(last_error.unwrap_or_else(|| BubblingError {
            source: VerbNounError::ResolvingError("All resolution strategies failed".to_string()),
            context: vec!["resolve_hybrid".to_string()],
            stress_level: 9.0,
            can_recover: false,
        }))
    }
}

// ==================== Diamond Dependency Prevention ====================

impl SemverXResolver {
    pub fn prevent_diamond_dependency(&mut self) -> Result<(), BubblingError> {
        // Detect strongly connected components (potential cycles)
        let sccs = tarjan_scc(&self.graph.graph);
        
        for scc in sccs {
            if scc.len() > 1 {
                // Found a cycle, break it with version pinning
                self.break_cycle_with_version_pinning(scc)?;
            }
        }
        
        Ok(())
    }
    
    fn break_cycle_with_version_pinning(&mut self, cycle: Vec<NodeIndex>) -> Result<(), BubblingError> {
        // Find the edge with highest stress/weight in the cycle
        let mut max_weight = 0.0;
        let mut break_edge = None;
        
        for i in 0..cycle.len() {
            let from = cycle[i];
            let to = cycle[(i + 1) % cycle.len()];
            
            if let Some(edge) = self.graph.graph.find_edge(from, to) {
                let weight = self.graph.graph[edge].weight;
                if weight > max_weight {
                    max_weight = weight;
                    break_edge = Some((from, to, edge));
                }
            }
        }
        
        // Remove the highest-stress edge to break the cycle
        if let Some((from, to, edge)) = break_edge {
            self.graph.graph.remove_edge(edge);
            
            // Pin the version of the target node
            if let Some(node) = self.graph.graph.node_weight_mut(to) {
                node.version.classifier = Some(crate::Classifier::Stable);
            }
        }
        
        Ok(())
    }
}

// ==================== Helper Structures ====================

#[derive(Debug)]
pub struct ResolutionResult {
    pub resolved_versions: HashMap<String, SemverX>,
    pub strategy_used: ResolutionStrategy,
    pub iterations: usize,
    pub stress_impact: f64,
}

#[derive(Debug)]
struct StressMonitor {
    samples: VecDeque<f64>,
    max_samples: usize,
}

impl StressMonitor {
    fn new() -> Self {
        Self {
            samples: VecDeque::new(),
            max_samples: 100,
        }
    }
    
    fn add_sample(&mut self, stress: f64) {
        if self.samples.len() >= self.max_samples {
            self.samples.pop_front();
        }
        self.samples.push_back(stress);
    }
    
    fn current_stress(&self) -> f64 {
        if self.samples.is_empty() {
            0.0
        } else {
            self.samples.iter().sum::<f64>() / self.samples.len() as f64
        }
    }
}

// ==================== Graph Operations ====================

impl DependencyGraph {
    fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
            stress_monitor: StressMonitor::new(),
        }
    }
    
    fn get_node(&self, id: &str) -> Result<NodeIndex, BubblingError> {
        self.node_map.get(id).copied()
            .ok_or_else(|| BubblingError {
                source: VerbNounError::ResolvingError(format!("Node not found: {}", id)),
                context: vec!["DependencyGraph::get_node".to_string()],
                stress_level: 3.0,
                can_recover: false,
            })
    }
    
    fn get_edges(&self, node: &NodeIndex) -> Vec<EdgeReference> {
        self.graph.edges(*node)
            .map(|e| EdgeReference {
                source: e.source(),
                target: e.target(),
                weight: e.weight().clone(),
            })
            .collect()
    }
    
    pub fn add_component(&mut self, component: ComponentNode) -> NodeIndex {
        let id = component.id.clone();
        let idx = self.graph.add_node(component);
        self.node_map.insert(id, idx);
        idx
    }
    
    pub fn add_dependency(&mut self, from: NodeIndex, to: NodeIndex, constraint: String, weight: f64) {
        self.graph.add_edge(from, to, DependencyEdge {
            constraint,
            weight,
            is_critical: weight > 5.0,
        });
    }
}

#[derive(Debug, Clone)]
struct EdgeReference {
    source: NodeIndex,
    target: NodeIndex,
    weight: DependencyEdge,
}

// ==================== Utility Functions ====================

impl SemverXResolver {
    fn validate_edge_compatibility(&self, from: &NodeIndex, edge: &EdgeReference) -> Result<(), BubblingError> {
        let from_node = self.graph.graph.node_weight(*from)
            .ok_or_else(|| BubblingError {
                source: VerbNounError::ResolvingError("Source node not found".to_string()),
                context: vec!["validate_edge_compatibility".to_string()],
                stress_level: 4.0,
                can_recover: false,
            })?;
        
        let to_node = self.graph.graph.node_weight(edge.target)
            .ok_or_else(|| BubblingError {
                source: VerbNounError::ResolvingError("Target node not found".to_string()),
                context: vec!["validate_edge_compatibility".to_string()],
                stress_level: 4.0,
                can_recover: false,
            })?;
        
        // Check version compatibility
        if !to_node.version.satisfies(&edge.weight.constraint)? {
            return Err(BubblingError {
                source: VerbNounError::ResolvingError(
                    format!("Version {} does not satisfy constraint {}", 
                            to_node.version.major, edge.weight.constraint)
                ),
                context: vec!["validate_edge_compatibility".to_string()],
                stress_level: 5.0,
                can_recover: true,
            });
        }
        
        Ok(())
    }
    
    fn is_resolution_complete(&self, visited: &HashSet<NodeIndex>) -> bool {
        // Check if we've resolved all critical dependencies
        for idx in visited {
            if let Some(node) = self.graph.graph.node_weight(*idx) {
                for dep in &node.dependencies {
                    if !dep.optional && !dep.dev {
                        // Check if this dependency is resolved
                        if !self.node_map.contains_key(&dep.name) {
                            return false;
                        }
                    }
                }
            }
        }
        true
    }
    
    fn extract_versions(&self, path: &[String]) -> HashMap<String, SemverX> {
        let mut versions = HashMap::new();
        
        for id in path {
            if let Some(idx) = self.node_map.get(id) {
                if let Some(node) = self.graph.graph.node_weight(*idx) {
                    versions.insert(node.id.clone(), node.version.clone());
                }
            }
        }
        
        versions
    }
    
    fn extract_versions_from_indices(&self, indices: &[NodeIndex]) -> HashMap<String, SemverX> {
        let mut versions = HashMap::new();
        
        for idx in indices {
            if let Some(node) = self.graph.graph.node_weight(*idx) {
                versions.insert(node.id.clone(), node.version.clone());
            }
        }
        
        versions
    }
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_strategy_selection() {
        let resolver = SemverXResolver::new();
        let strategy = resolver.determine_strategy_from_stress();
        assert_eq!(strategy, ResolutionStrategy::Hamiltonian);
    }
    
    #[test]
    fn test_stress_monitor() {
        let mut monitor = StressMonitor::new();
        monitor.add_sample(1.0);
        monitor.add_sample(2.0);
        monitor.add_sample(3.0);
        assert_eq!(monitor.current_stress(), 2.0);
    }
}