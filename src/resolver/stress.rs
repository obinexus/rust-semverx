// resolver/stress.rs - Stress monitoring for dependency resolution

use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct StressMonitor {
    stress_history: VecDeque<StressPoint>,
    max_history_size: usize,
    stress_threshold: f64,
    current_stress: f64,
}

#[derive(Debug, Clone)]
struct StressPoint {
    timestamp: Instant,
    value: f64,
    source: String,
}

impl StressMonitor {
    pub fn new() -> Self {
        Self {
            stress_history: VecDeque::new(),
            max_history_size: 100,
            stress_threshold: 8.0,
            current_stress: 0.0,
        }
    }
    
    pub fn current_stress(&self) -> f64 {
        self.current_stress
    }
    
    pub fn record_stress(&mut self, value: f64, source: String) {
        let point = StressPoint {
            timestamp: Instant::now(),
            value,
            source,
        };
        
        self.stress_history.push_back(point);
        
        // Maintain history size
        if self.stress_history.len() > self.max_history_size {
            self.stress_history.pop_front();
        }
        
        // Update current stress using exponential moving average
        self.current_stress = self.calculate_current_stress();
    }
    
    pub fn is_stressed(&self) -> bool {
        self.current_stress > self.stress_threshold
    }
    
    pub fn add_resolution_stress(&mut self, complexity: f64, iterations: u32) {
        let stress = complexity * (iterations as f64).ln();
        self.record_stress(stress, "resolution".to_string());
    }
    
    pub fn add_conflict_stress(&mut self, conflict_count: usize) {
        let stress = (conflict_count as f64) * 2.0;
        self.record_stress(stress, "conflict".to_string());
    }
    
    pub fn add_cycle_stress(&mut self, cycle_size: usize) {
        let stress = (cycle_size as f64).powf(1.5);
        self.record_stress(stress, "cycle".to_string());
    }
    
    fn calculate_current_stress(&self) -> f64 {
        if self.stress_history.is_empty() {
            return 0.0;
        }
        
        let now = Instant::now();
        let mut weighted_sum = 0.0;
        let mut weight_total = 0.0;
        
        // Use exponential decay for older stress points
        for point in &self.stress_history {
            let age = now.duration_since(point.timestamp).as_secs_f64();
            let weight = (-age / 60.0).exp(); // Decay over 1 minute
            
            weighted_sum += point.value * weight;
            weight_total += weight;
        }
        
        if weight_total > 0.0 {
            weighted_sum / weight_total
        } else {
            0.0
        }
    }
    
    pub fn reset(&mut self) {
        self.stress_history.clear();
        self.current_stress = 0.0;
    }
}
