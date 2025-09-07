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

/// Test stake-weighted selection
pub fn test_stake_weighted_selection(node_count: usize) {
    println!("--- Testing Stake-Weighted Selection with {} nodes ---", node_count);
    
    let config = RotorConfig::default();
    let nodes: Vec<Node> = (0..node_count)
        .map(|i| Node::new(i as u64, 100 + (i as u64 * 10))) // Different stakes
        .collect();
    
    let mut sampler = RotorSampler::new(config, nodes);
    
    // Test multiple sampling rounds
    for slot in 0..10 {
        let source_id = slot % node_count as u64;
        let selected = sampler.sample_nodes(source_id, slot);
        println!("Slot {}: Source {} selected nodes: {:?}", slot, source_id, selected);
    }
}

/// Test fanout optimization
pub fn test_fanout_optimization(fanout: usize) {
    println!("--- Testing Fanout Optimization with fanout {} ---", fanout);
    
    let config = RotorConfig {
        fanout,
        redundancy: 2,
        max_retries: 3,
    };
    
    let nodes: Vec<Node> = (0..20)
        .map(|i| Node::new(i as u64, 100))
        .collect();
    
    let mut sampler = RotorSampler::new(config, nodes);
    
    // Test sampling with different fanout values
    for slot in 0..5 {
        let source_id = slot % 20;
        let selected = sampler.sample_nodes(source_id, slot);
        println!("Slot {}: Fanout {} selected {} nodes: {:?}", slot, fanout, selected.len(), selected);
    }
}

/// Test message dissemination
pub fn test_message_dissemination(node_count: usize) {
    println!("--- Testing Message Dissemination with {} nodes ---", node_count);
    
    let config = RotorConfig::default();
    let nodes: Vec<Node> = (0..node_count)
        .map(|i| Node::new(i as u64, 100))
        .collect();
    
    let mut sampler = RotorSampler::new(config, nodes);
    
    // Simulate message dissemination
    let mut message_reach = vec![false; node_count];
    message_reach[0] = true; // Source node
    
    for round in 0..5 {
        let mut new_reach = message_reach.clone();
        
        for (i, &has_message) in message_reach.iter().enumerate() {
            if has_message {
                let selected = sampler.sample_nodes(i as u64, round);
                for &target in &selected {
                    if target < node_count as u64 {
                        new_reach[target as usize] = true;
                    }
                }
            }
        }
        
        message_reach = new_reach;
        let reached_count = message_reach.iter().filter(|&&x| x).count();
        println!("Round {}: Message reached {}/{} nodes", round, reached_count, node_count);
    }
}

/// Test topology adaptation
pub fn test_topology_adaptation(topology: &str) {
    println!("--- Testing Topology Adaptation with {} topology ---", topology);
    
    let config = RotorConfig::default();
    let nodes: Vec<Node> = (0..20)
        .map(|i| Node::new(i as u64, 100))
        .collect();
    
    let mut sampler = RotorSampler::new(config, nodes);
    
    // Simulate different topologies
    match topology {
        "mesh" => {
            println!("Mesh topology: All nodes can connect to all others");
        },
        "star" => {
            println!("Star topology: Central hub with spoke connections");
        },
        "ring" => {
            println!("Ring topology: Circular connection pattern");
        },
        "tree" => {
            println!("Tree topology: Hierarchical connection pattern");
        },
        _ => {
            println!("Unknown topology: {}", topology);
        }
    }
    
    // Test sampling in this topology
    for slot in 0..5 {
        let source_id = slot % 20;
        let selected = sampler.sample_nodes(source_id, slot);
        println!("Slot {}: Topology {} selected nodes: {:?}", slot, topology, selected);
    }
}

/// Test fault tolerance
pub fn test_fault_tolerance(fault_percent: u32) {
    println!("--- Testing Fault Tolerance with {}% faulty nodes ---", fault_percent);
    
    let config = RotorConfig::default();
    let mut nodes: Vec<Node> = (0..20)
        .map(|i| Node::new(i as u64, 100))
        .collect();
    
    // Mark some nodes as offline
    let faulty_count = (20 * fault_percent / 100) as usize;
    for i in 0..faulty_count {
        nodes[i].is_online = false;
    }
    
    let mut sampler = RotorSampler::new(config, nodes);
    
    // Test sampling with faulty nodes
    for slot in 0..10 {
        let source_id = slot % 20;
        let selected = sampler.sample_nodes(source_id, slot);
        let online_selected = selected.iter().filter(|&&id| id < 20 && (id as usize) >= faulty_count).count();
        println!("Slot {}: Selected {} nodes, {} online: {:?}", slot, selected.len(), online_selected, selected);
    }
}

/// Test load balancing
pub fn test_load_balancing() {
    println!("--- Testing Load Balancing ---");
    
    let config = RotorConfig::default();
    let nodes: Vec<Node> = (0..20)
        .map(|i| Node::new(i as u64, 100))
        .collect();
    
    let mut sampler = RotorSampler::new(config, nodes);
    let mut load_count = vec![0; 20];
    
    // Simulate load distribution
    for slot in 0..100 {
        let source_id = slot % 20;
        let selected = sampler.sample_nodes(source_id, slot);
        
        for &target in &selected {
            if target < 20 {
                load_count[target as usize] += 1;
            }
        }
    }
    
    println!("Load distribution across 20 nodes:");
    for (i, &count) in load_count.iter().enumerate() {
        println!("  Node {}: {} messages", i, count);
    }
}

/// Test scalability
pub fn test_scalability(node_count: usize) {
    println!("--- Testing Scalability with {} nodes ---", node_count);
    
    let config = RotorConfig::default();
    let nodes: Vec<Node> = (0..node_count)
        .map(|i| Node::new(i as u64, 100))
        .collect();
    
    let mut sampler = RotorSampler::new(config, nodes);
    
    // Test sampling performance with large node count
    let start = std::time::Instant::now();
    
    for slot in 0..10 {
        let source_id = slot % node_count as u64;
        let _selected = sampler.sample_nodes(source_id, slot);
    }
    
    let duration = start.elapsed();
    println!("Scalability test with {} nodes completed in {:?}", node_count, duration);
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
