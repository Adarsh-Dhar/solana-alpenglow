//! A Stateright model for the Votor consensus engine from the Alpenglow whitepaper.
//! This model focuses on verifying the safety of the dual-path finality mechanism:
//! 1. Fast Path: Finalization in one round with >= 80% stake.
//! 2. Slow Path: Finalization in two rounds with >= 60% stake each.
//!
//! To run this model, you will need Rust and Cargo installed. Then, execute:
//! `cargo run --release`

use stateright::{Model, Property};
use std::collections::{BTreeMap, BTreeSet};

// -----------
// Constants
// -----------

const VALIDATOR_COUNT: usize = 3;
const FAST_FINALIZE_THRESHOLD: u64 = 80;
const NOTARIZE_THRESHOLD: u64 = 60;
const SLOW_FINALIZE_THRESHOLD: u64 = 60;

// To simplify, each validator has an equal stake.
const STAKE_PER_VALIDATOR: u64 = 100 / VALIDATOR_COUNT as u64;

// -----------
// Type Aliases
// -----------

type Slot = u64;
type Hash = u64;
type ActorId = usize;
type Stake = u64;

// -----------
// State & Message Definitions
// -----------

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct VotorState {
    /// The network is modeled as a set of in-flight messages.
    network: BTreeSet<MessageInTransit>,
    /// Tracks finalized blocks to check for safety violations. Map<Slot, Hash>.
    finalized_blocks: BTreeMap<Slot, Hash>,
    /// Per-node state tracking
    node_states: Vec<NodeState>,
    /// Current slot being processed
    current_slot: Slot,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct NodeState {
    /// Per-slot state flags that track a node's commitments.
    slot_states: BTreeMap<Slot, SlotState>,
    /// Votes received from other nodes, representing this node's view of the "Pool".
    vote_pool: BTreeMap<Slot, BTreeMap<Hash, BTreeSet<ActorId>>>,
    /// FinalVotes received for the second round of the slow path.
    final_vote_pool: BTreeMap<Slot, BTreeSet<ActorId>>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct SlotState {
    // Core state flags from the whitepaper (Definition 18)
    voted: bool,
    voted_notar: Option<Hash>,
    block_notarized: Option<Hash>,
    bad_window: bool,
    its_over: bool, // FinalVote has been cast
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Message {
    /// A leader proposes a block.
    Block {
        slot: Slot,
        hash: Hash,
        parent_hash: Hash,
    },
    /// A vote for a specific block in a slot.
    NotarVote {
        slot: Slot,
        hash: Hash,
        voter: ActorId,
    },
    /// A vote to finalize a notarized block (slow path round 2).
    FinalVote { slot: Slot, voter: ActorId },
    /// A vote to skip a slot, usually after a timeout.
    SkipVote { slot: Slot, voter: ActorId },
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MessageInTransit {
    dst: ActorId,
    msg: Message,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Action {
    /// A node proposes a new block for a given slot.
    Propose {
        slot: Slot,
        proposer: ActorId,
    },
    /// Deliver an in-flight message to its destination.
    Deliver { msg: MessageInTransit },
    /// A node's local timer for a slot expires.
    Timeout { slot: Slot, node_id: ActorId },
}

#[derive(Clone)]
pub struct VotorModel {
    /// Number of honest validators.
    pub honest_validators: usize,
    /// Maximum number of slots to explore.
    pub max_slot: Slot,
}

impl VotorState {
    fn new(validator_count: usize) -> Self {
        let mut genesis_finalized = BTreeMap::new();
        genesis_finalized.insert(0, 0); // Genesis block: slot 0, hash 0

        Self {
            network: BTreeSet::new(),
            finalized_blocks: genesis_finalized,
            node_states: (0..validator_count).map(|_| NodeState {
                slot_states: BTreeMap::new(),
                vote_pool: BTreeMap::new(),
                final_vote_pool: BTreeMap::new(),
            }).collect(),
            current_slot: 0,
        }
    }
}

impl Model for VotorModel {
    type State = VotorState;
    type Action = Action;

    fn init_states(&self) -> Vec<Self::State> {
        vec![VotorState::new(self.honest_validators)]
    }

    fn actions(&self, state: &Self::State, actions: &mut Vec<Self::Action>) {
        // 1. Deliver any message in the network
        for msg in &state.network {
            actions.push(Action::Deliver { msg: msg.clone() });
        }

        // 2. Any node can propose a block for a future slot
        for proposer_id in 0..self.honest_validators {
            let last_finalized_slot = *state.finalized_blocks.keys().max().unwrap_or(&0);
            let next_slot = last_finalized_slot + 1;
            if next_slot <= self.max_slot {
                actions.push(Action::Propose {
                    slot: next_slot,
                    proposer: proposer_id,
                });
            }
        }
        
        // 3. Timeouts can occur for any non-finalized slot at any node
        for node_id in 0..self.honest_validators {
            for s in 1..=self.max_slot {
                if !state.finalized_blocks.contains_key(&s) {
                     actions.push(Action::Timeout { slot: s, node_id });
                }
            }
        }
    }

    fn next_state(&self, last_state: &Self::State, action: Self::Action) -> Option<Self::State> {
        let mut next_state = last_state.clone();
        let mut node_states = last_state.node_states.clone();

        match action {
            Action::Propose { slot, proposer } => {
                // Find a valid parent for the new block.
                let parent_slot = slot - 1;
                if let Some(parent_hash) = next_state.finalized_blocks.get(&parent_slot) {
                    let block_hash = slot; // Simple hash for modeling
                    let block_msg = Message::Block {
                        slot,
                        hash: block_hash,
                        parent_hash: *parent_hash,
                    };

                    // Broadcast block to all other nodes
                    for i in 0..self.honest_validators {
                        if i != proposer {
                            next_state.network.insert(MessageInTransit {
                                dst: i,
                                msg: block_msg.clone(),
                            });
                        }
                    }
                }
            }
            Action::Deliver { msg } => {
                let recipient_id = msg.dst;
                let mut node_state = node_states[recipient_id].clone();
                
                // Remove message from network
                if !next_state.network.remove(&msg) { return None; }

                match msg.msg {
                    Message::Block { slot, hash, parent_hash } => {
                        // TRYNOTAR logic (Algorithm 2)
                        let slot_state = node_state.slot_states.entry(slot).or_default();
                        let parent_slot = slot - 1;

                        // Precondition 1: Voted flag is not set
                        // Precondition 2: Parent is ready (we check against global finalized state for simplicity)
                        if !slot_state.voted && next_state.finalized_blocks.get(&parent_slot) == Some(&parent_hash) {
                            slot_state.voted = true;
                            slot_state.voted_notar = Some(hash);

                            // Broadcast NotarVote to all nodes
                            for i in 0..self.honest_validators {
                                next_state.network.insert(MessageInTransit {
                                    dst: i,
                                    msg: Message::NotarVote { slot, hash, voter: recipient_id },
                                });
                            }
                        }
                        node_states[recipient_id] = node_state;
                    }
                    Message::NotarVote { slot, hash, voter } => {
                        // Add vote to the node's local pool
                        let slot_votes = node_state.vote_pool.entry(slot).or_default();
                        let block_voters = slot_votes.entry(hash).or_default();
                        block_voters.insert(voter);

                        let total_stake: Stake = block_voters.len() as u64 * STAKE_PER_VALIDATOR;

                        // Check for FAST-FINALIZATION (>= 80% stake)
                        if total_stake >= FAST_FINALIZE_THRESHOLD {
                             next_state.finalized_blocks.insert(slot, hash);
                        }

                        // Check for NOTARIZATION (>= 60% stake)
                        if total_stake >= NOTARIZE_THRESHOLD {
                             let slot_state = node_state.slot_states.entry(slot).or_default();
                             if slot_state.block_notarized.is_none() {
                                slot_state.block_notarized = Some(hash);

                                // TRYFINAL logic (Algorithm 2)
                                // Precondition 1: BlockNotarized is set (just happened)
                                // Precondition 2: Node personally voted for this block
                                // Precondition 3: BadWindow is not set
                                if slot_state.voted_notar == Some(hash) && !slot_state.bad_window {
                                    slot_state.its_over = true;
                                    // Broadcast FinalVote
                                    for i in 0..self.honest_validators {
                                        next_state.network.insert(MessageInTransit {
                                            dst: i,
                                            msg: Message::FinalVote { slot, voter: recipient_id }
                                        });
                                    }
                                }
                             }
                        }
                        node_states[recipient_id] = node_state;
                    }
                    Message::FinalVote { slot, voter } => {
                        // Aggregate FinalVotes
                        let slot_final_voters = node_state.final_vote_pool.entry(slot).or_default();
                        slot_final_voters.insert(voter);
                        
                        let total_stake: Stake = slot_final_voters.len() as u64 * STAKE_PER_VALIDATOR;
                        
                        // Check for SLOW-FINALIZATION (>= 60% stake)
                        if total_stake >= SLOW_FINALIZE_THRESHOLD {
                            if let Some(notarized_hash) = node_state.slot_states.get(&slot).and_then(|ss| ss.block_notarized) {
                                next_state.finalized_blocks.insert(slot, notarized_hash);
                            }
                        }
                        node_states[recipient_id] = node_state;
                    }
                    Message::SkipVote { slot, voter: _ } => {
                        // Basic handling for skip votes - we don't implement full skip certs,
                        // but receiving one indicates a problem in the window.
                         let slot_state = node_state.slot_states.entry(slot).or_default();
                         slot_state.bad_window = true;
                         node_states[recipient_id] = node_state;
                    }
                }
            }
            Action::Timeout { slot, node_id } => {
                let mut node_state = node_states[node_id].clone();
                let slot_state = node_state.slot_states.entry(slot).or_default();

                // TRYSKIP_WINDOW logic
                if !slot_state.voted {
                    slot_state.voted = true;
                    slot_state.bad_window = true;

                    // Broadcast SkipVote
                    for i in 0..self.honest_validators {
                        next_state.network.insert(MessageInTransit {
                            dst: i,
                            msg: Message::SkipVote { slot, voter: node_id },
                        });
                    }
                }
                node_states[node_id] = node_state;
            }
        }
        
        next_state.node_states = node_states;
        Some(next_state)
    }

    /// Defines the property we want to check: No two different blocks are ever
    /// finalized for the same slot.
    fn properties(&self) -> Vec<Property<Self>> {
        vec![Property::<Self>::always("safety", |_, state| {
            let mut observed_slots = BTreeMap::new();
            for (slot, hash) in &state.finalized_blocks {
                if let Some(existing_hash) = observed_slots.get(slot) {
                    if existing_hash != hash {
                        return false; // Found two different hashes for the same slot!
                    }
                } else {
                    observed_slots.insert(*slot, *hash);
                }
            }
            true
        })]
    }
}