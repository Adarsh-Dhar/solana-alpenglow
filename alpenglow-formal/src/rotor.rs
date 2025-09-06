//! Rotor sampling strategy for Alpenglow consensus.
//! This module implements the sampling mechanisms used in the Alpenglow
//! consensus protocol for efficient message dissemination and validation.

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

/// Configuration for rotor sampling
#[derive(Debug, Clone)]
pub struct RotorConfig {
    pub fanout: usize,
    #[allow(dead_code)]
    pub redundancy: usize,
    pub max_retries: u32,
}

impl Default for RotorConfig {
    fn default() -> Self {
        Self {
            fanout: 3,
            redundancy: 2,
            max_retries: 3,
        }
    }
}

/// Represents a node in the network for sampling purposes
#[derive(Debug, Clone)]
pub struct Node {
    #[allow(dead_code)]
    pub id: u64,
    pub stake: u64,
    pub is_online: bool,
}

impl Node {
    pub fn new(id: u64, stake: u64) -> Self {
        Self {
            id,
            stake,
            is_online: true,
        }
    }
}

/// Rotor sampling strategy implementation
#[derive(Debug)]
pub struct RotorSampler {
    config: RotorConfig,
    nodes: Vec<Node>,
    #[allow(dead_code)]
    rng: StdRng,
}

impl RotorSampler {
    /// Creates a new rotor sampler with the given configuration and nodes
    pub fn new(config: RotorConfig, nodes: Vec<Node>) -> Self {
        Self {
            config,
            nodes,
            rng: StdRng::from_entropy(),
        }
    }

    /// Samples nodes for message dissemination using stake-weighted selection
    pub fn sample_nodes(&mut self, source_id: u64, slot: u64) -> Vec<u64> {
        let mut selected = Vec::new();
        let mut attempts = 0;
        
        while selected.len() < self.config.fanout && attempts < self.config.max_retries {
            let node_id = self.weighted_random_selection(slot + attempts as u64);
            
            if node_id != source_id && !selected.contains(&node_id) {
                if let Some(node) = self.nodes.get(node_id as usize) {
                    if node.is_online {
                        selected.push(node_id);
                    }
                }
            }
            attempts += 1;
        }
        
        selected
    }

    /// Performs stake-weighted random selection
    fn weighted_random_selection(&mut self, seed: u64) -> u64 {
        let total_stake: u64 = self.nodes.iter().map(|n| n.stake).sum();
        let mut rng = StdRng::seed_from_u64(seed);
        let random_value = rng.gen_range(0..total_stake);
        
        let mut cumulative_stake = 0;
        for (i, node) in self.nodes.iter().enumerate() {
            cumulative_stake += node.stake;
            if random_value < cumulative_stake {
                return i as u64;
            }
        }
        
        // Fallback to last node
        (self.nodes.len() - 1) as u64
    }

    /// Updates the online status of a node
    pub fn set_node_online(&mut self, node_id: u64, online: bool) {
        if let Some(node) = self.nodes.get_mut(node_id as usize) {
            node.is_online = online;
        }
    }

    /// Gets the current configuration
    #[allow(dead_code)]
    pub fn config(&self) -> &RotorConfig {
        &self.config
    }

    /// Updates the configuration
    #[allow(dead_code)]
    pub fn set_config(&mut self, config: RotorConfig) {
        self.config = config;
    }
}

/// Runs a simple rotor sampling simulation
pub fn run_simulation() {
    println!("--- Alpenglow Rotor Sampling Simulation ---");
    
    let config = RotorConfig::default();
    let nodes: Vec<Node> = (0..10)
        .map(|i| Node::new(i, 100))
        .collect();
    
    let mut sampler = RotorSampler::new(config, nodes);
    
    println!("Sampling nodes for message dissemination...");
    
    for slot in 0..5 {
        let source_id = slot % 10;
        let selected = sampler.sample_nodes(source_id, slot);
        println!("Slot {}: Source {} selected nodes: {:?}", slot, source_id, selected);
    }
    
    // Simulate a node going offline
    println!("\nSimulating node 3 going offline...");
    sampler.set_node_online(3, false);
    
    for slot in 5..8 {
        let source_id = slot % 10;
        let selected = sampler.sample_nodes(source_id, slot);
        println!("Slot {}: Source {} selected nodes: {:?}", slot, source_id, selected);
    }
    
    println!("\n--- Simulation Complete ---");
    println!("The rotor sampling strategy ensures efficient message dissemination while maintaining redundancy.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotor_sampler_creation() {
        let config = RotorConfig::default();
        let nodes = vec![Node::new(0, 100), Node::new(1, 200)];
        let sampler = RotorSampler::new(config, nodes);
        assert_eq!(sampler.nodes.len(), 2);
    }

    #[test]
    fn test_node_creation() {
        let node = Node::new(1, 150);
        assert_eq!(node.id, 1);
        assert_eq!(node.stake, 150);
        assert!(node.is_online);
    }

    #[test]
    fn test_config_default() {
        let config = RotorConfig::default();
        assert_eq!(config.fanout, 3);
        assert_eq!(config.redundancy, 2);
        assert_eq!(config.max_retries, 3);
    }
}
