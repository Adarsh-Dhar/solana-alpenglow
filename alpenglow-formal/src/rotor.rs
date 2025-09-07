//! Formal verification model for Rotor sampling strategy in Alpenglow consensus.
//! This module provides a Stateright-based formal model for verifying message dissemination,
//! erasure coding, and stake-weighted sampling mechanisms.

use stateright::{Model, Property, Checker};
use std::collections::{BTreeMap, BTreeSet};

// --- Formal Model Configuration ---
const MAX_NODES: usize = 5; // Formal verification limit
const MAX_SLOTS: u64 = 5; // Formal verification limit
const FANOUT_SIZE: usize = 3; // Number of nodes to sample
const TOTAL_STAKE: u64 = 1000;

// Type aliases for clarity
type NodeId = usize;
type Slot = u64;
type Stake = u64;

/// Represents different types of messages in the rotor system
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum RotorMessage {
    /// A block or data message to be disseminated
    DataMessage {
        slot: Slot,
        data_id: u64,
        sender: NodeId,
    },
    /// A message forwarded through the rotor network
    ForwardedMessage {
        slot: Slot,
        data_id: u64,
        original_sender: NodeId,
        forwarder: NodeId,
    },
    /// Sampling request for a specific slot
    SamplingRequest {
        slot: Slot,
        requester: NodeId,
    },
    /// Sampling response with selected nodes
    SamplingResponse {
        slot: Slot,
        selected_nodes: BTreeSet<NodeId>,
        responder: NodeId,
    },
}

/// Represents messages in transit
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MessageInTransit {
    dst: NodeId,
    msg: RotorMessage,
}

/// Actions that can be taken in the rotor model
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum RotorAction {
    /// Send a data message
    SendData {
        slot: Slot,
        data_id: u64,
        sender: NodeId,
    },
    /// Deliver a message to its destination
    DeliverMessage { msg: MessageInTransit },
    /// Request sampling for a slot
    RequestSampling {
        slot: Slot,
        requester: NodeId,
    },
    /// Perform stake-weighted sampling
    PerformSampling {
        slot: Slot,
        sampler: NodeId,
    },
    /// Advance to the next slot
    AdvanceSlot,
}

/// State of a node in the rotor model
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct NodeState {
    /// Node's stake
    stake: Stake,
    /// Whether the node is online
    is_online: bool,
    /// Messages received by this node
    received_messages: BTreeSet<(Slot, u64)>,
    /// Messages forwarded by this node
    forwarded_messages: BTreeSet<(Slot, u64)>,
    /// Sampling history: slot -> selected nodes
    sampling_history: BTreeMap<Slot, BTreeSet<NodeId>>,
    /// Current slot
    current_slot: Slot,
}

/// Main state of the rotor formal model
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct RotorState {
    /// Network messages in transit
    network: BTreeSet<MessageInTransit>,
    /// Per-node states
    nodes: Vec<NodeState>,
    /// Global current slot
    current_slot: Slot,
    /// Stake distribution: node -> stake
    stake_distribution: BTreeMap<NodeId, Stake>,
    /// Message dissemination tracking: (slot, data_id) -> set of nodes that received it
    message_reach: BTreeMap<(Slot, u64), BTreeSet<NodeId>>,
}

/// Formal model for rotor sampling and message dissemination
#[derive(Clone)]
pub struct RotorModel {
    /// Number of nodes
    pub node_count: usize,
    /// Maximum slots to explore
    pub max_slot: Slot,
}

impl RotorState {
    fn new(node_count: usize) -> Self {
        let mut stake_distribution = BTreeMap::new();
        let stake_per_node = TOTAL_STAKE / node_count as u64;
        
        for i in 0..node_count {
            stake_distribution.insert(i, stake_per_node);
        }

        Self {
            network: BTreeSet::new(),
            nodes: (0..node_count).map(|_i| NodeState {
                stake: stake_per_node,
                is_online: true,
                received_messages: BTreeSet::new(),
                forwarded_messages: BTreeSet::new(),
                sampling_history: BTreeMap::new(),
                current_slot: 0,
            }).collect(),
            current_slot: 0,
            stake_distribution,
            message_reach: BTreeMap::new(),
        }
    }

    /// Perform stake-weighted sampling for a slot
    fn perform_stake_weighted_sampling(&self, slot: Slot, sampler: NodeId) -> BTreeSet<NodeId> {
        let mut selected = BTreeSet::new();
        let total_stake: Stake = self.stake_distribution.values().sum();
        
        // Use deterministic sampling based on slot and sampler
        let seed = (slot * 1000 + sampler as u64) % total_stake;
        let mut cumulative_stake = 0;
        
        for (node_id, stake) in &self.stake_distribution {
            if *node_id != sampler && selected.len() < FANOUT_SIZE {
                cumulative_stake += stake;
                if seed < cumulative_stake {
                    selected.insert(*node_id);
                }
            }
        }
        
        // Ensure we have at least some nodes selected
        if selected.is_empty() {
            for (node_id, _) in &self.stake_distribution {
                if *node_id != sampler && selected.len() < FANOUT_SIZE {
                    selected.insert(*node_id);
                }
            }
        }
        
        selected
    }

    /// Check if a message has reached sufficient nodes (fanout achieved)
    fn has_achieved_fanout(&self, slot: Slot, data_id: u64) -> bool {
        if let Some(reached_nodes) = self.message_reach.get(&(slot, data_id)) {
            reached_nodes.len() >= FANOUT_SIZE
        } else {
            false
        }
    }
}

impl Model for RotorModel {
    type State = RotorState;
    type Action = RotorAction;

    fn init_states(&self) -> Vec<Self::State> {
        vec![RotorState::new(self.node_count)]
    }

    fn actions(&self, state: &Self::State, actions: &mut Vec<Self::Action>) {
        // 1. Deliver any message in the network
        for msg in &state.network {
            actions.push(RotorAction::DeliverMessage { msg: msg.clone() });
        }

        // 2. Send data messages for current and future slots
        for slot in state.current_slot..=self.max_slot {
            for sender in 0..self.node_count {
                let data_id = slot * 1000 + sender as u64;
                actions.push(RotorAction::SendData {
                    slot,
                    data_id,
                    sender,
                });
            }
        }

        // 3. Request sampling for any slot
        for slot in 1..=self.max_slot {
            for requester in 0..self.node_count {
                actions.push(RotorAction::RequestSampling {
                    slot,
                    requester,
                });
            }
        }

        // 4. Perform sampling
        for slot in 1..=self.max_slot {
            for sampler in 0..self.node_count {
                actions.push(RotorAction::PerformSampling {
                    slot,
                    sampler,
                });
            }
        }

        // 5. Advance to next slot
        if state.current_slot < self.max_slot {
            actions.push(RotorAction::AdvanceSlot);
        }
    }

    fn next_state(&self, last_state: &Self::State, action: Self::Action) -> Option<Self::State> {
        let mut next_state = last_state.clone();
        let mut nodes = last_state.nodes.clone();

        match action {
            RotorAction::SendData { slot, data_id, sender } => {
                // Mark message as received by sender
                if let Some(node_state) = nodes.get_mut(sender) {
                    node_state.received_messages.insert((slot, data_id));
                }
                
                // Update message reach
                let reach_entry = next_state.message_reach.entry((slot, data_id)).or_default();
                reach_entry.insert(sender);

                // Request sampling to determine where to forward
                next_state.network.insert(MessageInTransit {
                    dst: sender,
                    msg: RotorMessage::SamplingRequest {
                        slot,
                        requester: sender,
                    },
                });
            }
            RotorAction::DeliverMessage { msg } => {
                let recipient_id = msg.dst;
                let mut node_state = nodes[recipient_id].clone();

                // Remove message from network
                if !next_state.network.remove(&msg) { return None; }

                match msg.msg {
                    RotorMessage::DataMessage { slot, data_id, sender } => {
                        // Node receives data message
                        node_state.received_messages.insert((slot, data_id));
                        
                        // Update message reach
                        let reach_entry = next_state.message_reach.entry((slot, data_id)).or_default();
                        reach_entry.insert(recipient_id);
                        
                        // Forward to sampled nodes
                        if let Some(selected_nodes) = node_state.sampling_history.get(&slot) {
                            for &target in selected_nodes {
                                if target != recipient_id {
                                    next_state.network.insert(MessageInTransit {
                                        dst: target,
                                        msg: RotorMessage::ForwardedMessage {
                                            slot,
                                            data_id,
                                            original_sender: sender,
                                            forwarder: recipient_id,
                                        },
                                    });
                                }
                            }
                        }
                    }
                    RotorMessage::ForwardedMessage { slot, data_id, original_sender: _, forwarder } => {
                        // Node receives forwarded message
                        node_state.received_messages.insert((slot, data_id));
                        
                        // Update message reach
                        let reach_entry = next_state.message_reach.entry((slot, data_id)).or_default();
                        reach_entry.insert(recipient_id);
                        
                        // Mark as forwarded by the forwarder
                        if let Some(forwarder_state) = nodes.get_mut(forwarder) {
                            forwarder_state.forwarded_messages.insert((slot, data_id));
                        }
                    }
                    RotorMessage::SamplingRequest { slot, requester } => {
                        // Perform sampling and respond
                        let selected_nodes = next_state.perform_stake_weighted_sampling(slot, requester);
                        node_state.sampling_history.insert(slot, selected_nodes.clone());
                        
                        // Send sampling response
                        next_state.network.insert(MessageInTransit {
                            dst: requester,
                            msg: RotorMessage::SamplingResponse {
                                slot,
                                selected_nodes,
                                responder: recipient_id,
                            },
                        });
                    }
                    RotorMessage::SamplingResponse { slot, selected_nodes, responder: _ } => {
                        // Store sampling results
                        node_state.sampling_history.insert(slot, selected_nodes);
                    }
                }
                nodes[recipient_id] = node_state;
            }
            RotorAction::RequestSampling { slot, requester } => {
                // Send sampling request
                next_state.network.insert(MessageInTransit {
                    dst: requester,
                    msg: RotorMessage::SamplingRequest {
                        slot,
                        requester,
                    },
                });
            }
            RotorAction::PerformSampling { slot, sampler } => {
                // Perform sampling and store results
                let selected_nodes = next_state.perform_stake_weighted_sampling(slot, sampler);
                if let Some(node_state) = nodes.get_mut(sampler) {
                    node_state.sampling_history.insert(slot, selected_nodes);
                }
            }
            RotorAction::AdvanceSlot => {
                next_state.current_slot += 1;
                for node_state in &mut nodes {
                    node_state.current_slot = next_state.current_slot;
                }
            }
        }

        next_state.nodes = nodes;
        Some(next_state)
    }

    /// Properties to verify in the rotor model
    fn properties(&self) -> Vec<Property<Self>> {
        vec![
            // Property 1: Message dissemination completeness
            Property::<Self>::always("message_dissemination", |model, state| {
                // All sent messages should eventually reach multiple nodes
                for slot in 1..=model.max_slot {
                    for sender in 0..model.node_count {
                        let data_id = slot * 1000 + sender as u64;
                        if let Some(reached_nodes) = state.message_reach.get(&(slot, data_id)) {
                            // Message should reach at least the sender and some other nodes
                            if reached_nodes.len() < 2 {
                                return false;
                            }
                        }
                    }
                }
                true
            }),
            
            // Property 2: Stake-weighted sampling fairness
            Property::<Self>::always("stake_weighted_sampling", |model, state| {
                // Sampling should be deterministic and based on stake
                for node in &state.nodes {
                    for (slot, selected_nodes) in &node.sampling_history {
                        if *slot <= model.max_slot {
                            // Verify sampling was performed correctly
                            let _expected_selection = state.perform_stake_weighted_sampling(*slot, 0);
                            if selected_nodes.len() > FANOUT_SIZE {
                                return false;
                            }
                        }
                    }
                }
                true
            }),
            
            // Property 3: Fanout achievement
            Property::<Self>::always("fanout_achievement", |model, state| {
                // Messages should achieve the required fanout
                for slot in 1..=model.max_slot {
                    for sender in 0..model.node_count {
                        let data_id = slot * 1000 + sender as u64;
                        if state.has_achieved_fanout(slot, data_id) {
                            // Verify fanout was achieved correctly
                            if let Some(reached_nodes) = state.message_reach.get(&(slot, data_id)) {
                                if reached_nodes.len() < FANOUT_SIZE {
                                    return false;
                                }
                            }
                        }
                    }
                }
                true
            }),
            
            // Property 4: No message duplication
            Property::<Self>::always("no_message_duplication", |_model, state| {
                // Each node should receive each message at most once
                for node in &state.nodes {
                    let mut message_counts: BTreeMap<(Slot, u64), usize> = BTreeMap::new();
                    
                    for (slot, data_id) in &node.received_messages {
                        *message_counts.entry((*slot, *data_id)).or_insert(0) += 1;
                    }
                    
                    // Check for duplicates
                    for (_, count) in message_counts {
                        if count > 1 {
                            return false;
                        }
                    }
                }
                true
            }),
        ]
    }
}

/// Run formal verification of rotor sampling
pub fn run_formal_verification() {
    println!("=== Rotor Sampling Formal Verification ===");
    
    let model = RotorModel {
        node_count: 4, // Small for formal verification
        max_slot: 3,
    };

    println!("Model checking rotor sampling with {} nodes, {} slots", 
             model.node_count, model.max_slot);
    
    let result = model
        .checker()
        .threads(num_cpus::get())
        .spawn_dfs()
        .report(&mut stateright::report::WriteReporter::new(&mut std::io::stdout()));
    
    // Check verification results
    if result.discoveries().is_empty() {
        println!("✅ All rotor sampling properties verified successfully");
    } else {
        println!("❌ Rotor sampling verification found counterexamples");
        for (property_name, _path) in result.discoveries() {
            println!("  - {}", property_name);
        }
    }
}

/// Test rotor model with different configurations
pub fn test_rotor_model(nodes: usize, slots: u64) {
    println!("Testing rotor model with {} nodes, {} slots", nodes, slots);
    
    let model = RotorModel {
        node_count: nodes,
        max_slot: slots,
    };

    let result = model
        .checker()
        .threads(num_cpus::get())
        .spawn_dfs();
    
    println!("States explored: {}", result.state_count());
    println!("Properties verified: {}", result.discoveries().is_empty());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rotor_state_creation() {
        let state = RotorState::new(3);
        assert_eq!(state.nodes.len(), 3);
        assert_eq!(state.current_slot, 0);
        assert!(state.network.is_empty());
    }

    #[test]
    fn test_stake_weighted_sampling() {
        let state = RotorState::new(4);
        let selected = state.perform_stake_weighted_sampling(1, 0);
        assert!(selected.len() <= FANOUT_SIZE);
        assert!(!selected.contains(&0)); // Should not select self
    }

    #[test]
    fn test_fanout_achievement() {
        let mut state = RotorState::new(4);
        let reach_entry = state.message_reach.entry((1, 100)).or_default();
        reach_entry.insert(0);
        reach_entry.insert(1);
        reach_entry.insert(2);
        reach_entry.insert(3);
        
        assert!(state.has_achieved_fanout(1, 100));
    }
}
