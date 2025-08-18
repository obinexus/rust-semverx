// resolver/strategies.rs - Resolution strategy definitions

use crate::resolver::types::Component;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum ResolutionStrategy {
    Eulerian,    // Visit all edges (comprehensive)
    Hamiltonian, // Direct node path (fast)
    AStar,       // Nearest viable path (optimal)
    Hybrid,      // Adaptive based on stress
}

#[derive(Debug, Clone)]
pub struct ResolutionResult {
    pub resolved_components: Vec<Component>,
    pub resolution_path: Vec<String>,
    pub conflicts_resolved: usize,
    pub iterations: u32,
    pub strategy_used: ResolutionStrategy,
    pub stress_level: f64,
}

impl ResolutionResult {
    pub fn new(strategy: ResolutionStrategy) -> Self {
        Self {
            resolved_components: Vec::new(),
            resolution_path: Vec::new(),
            conflicts_resolved: 0,
            iterations: 0,
            strategy_used: strategy,
            stress_level: 0.0,
        }
    }
    
    pub fn add_component(&mut self, component: Component) {
        self.resolution_path.push(component.id.clone());
        self.resolved_components.push(component);
    }
    
    pub fn increment_iterations(&mut self) {
        self.iterations += 1;
    }
    
    pub fn record_conflict_resolution(&mut self) {
        self.conflicts_resolved += 1;
    }
    
    pub fn set_stress_level(&mut self, stress: f64) {
        self.stress_level = stress;
    }
}

// A* pathfinding node for resolution
#[derive(Debug, Clone)]
pub struct AStarNode {
    pub component_id: String,
    pub g_cost: f64, // Cost from start
    pub h_cost: f64, // Heuristic cost to goal
    pub f_cost: f64, // Total cost (g + h)
    pub parent: Option<String>,
}

impl AStarNode {
    pub fn new(component_id: String, g_cost: f64, h_cost: f64) -> Self {
        Self {
            component_id,
            g_cost,
            h_cost,
            f_cost: g_cost + h_cost,
            parent: None,
        }
    }
}

impl Eq for AStarNode {}

impl PartialEq for AStarNode {
    fn eq(&self, other: &Self) -> bool {
        self.component_id == other.component_id
    }
}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse ordering for min-heap behavior
        other.f_cost.partial_cmp(&self.f_cost)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
