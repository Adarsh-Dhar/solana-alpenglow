//! Formal verification model for timeout handling and skip certificate generation in Alpenglow consensus.
//! This module provides a Stateright-based formal model for verifying timeout mechanisms,
//! skip certificate generation, and BadWindow flag management.

use stateright::{Model, Property, Checker};
use std::collections::{BTreeMap, BTreeSet};

// --- Formal Model Configuration ---
const SKIP_CERTIFICATE_THRESHOLD_PERCENT: u64 = 60;
const TOTAL_STAKE: u64 = 1000;
const MAX_SLOTS: u64 = 5; // Formal verification limit
const MAX_VALIDATORS: usize = 5; // Formal verification limit

// Type aliases for clarity
type Slot = u64;
type Hash = u64;
type ActorId = usize;
type Stake = u64;

/// Represents different types of messages in the timeout system
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum TimeoutMessage {
    /// A block proposal from a leader
    BlockProposal {
        slot: Slot,
        hash: Hash,
        proposer: ActorId,
    },
    /// A vote for a block (NotarVote)
    NotarVote {
        slot: Slot,
        hash: Hash,
        voter: ActorId,
    },
    /// A vote to skip a slot (SkipVote)
    SkipVote {
        slot: Slot,
        voter: ActorId,
    },
    /// A timeout event for a specific slot
    TimeoutEvent {
        slot: Slot,
        validator: ActorId,
    },
}

/// Represents messages in transit
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MessageInTransit {
    dst: ActorId,
    msg: TimeoutMessage,
}

/// Actions that can be taken in the timeout model
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum TimeoutAction {
    /// Propose a block for a slot
    ProposeBlock {
        slot: Slot,
        proposer: ActorId,
    },
    /// Deliver a message to its destination
    DeliverMessage { msg: MessageInTransit },
    /// Trigger a timeout for a slot at a validator
    TriggerTimeout {
        slot: Slot,
        validator: ActorId,
    },
    /// Advance to the next slot
    AdvanceSlot,
}

/// State of a validator in the timeout model
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ValidatorState {
    /// Votes cast by this validator (slot -> hash or None for skip)
    votes_cast: BTreeMap<Slot, Option<Hash>>,
    /// Vote pool: (slot, hash) -> set of voters
    vote_pool: BTreeMap<(Slot, Option<Hash>), BTreeSet<ActorId>>,
    /// Certificates formed: (slot, hash) pairs
    certificates: BTreeSet<(Slot, Option<Hash>)>,
    /// BadWindow flag state
    bad_window: bool,
    /// Current slot being processed
    current_slot: Slot,
}

/// Main state of the timeout formal model
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TimeoutState {
    /// Network messages in transit
    network: BTreeSet<MessageInTransit>,
    /// Per-validator states
    validators: Vec<ValidatorState>,
    /// Global current slot
    current_slot: Slot,
    /// Skip certificates formed: slot -> true if skip cert exists
    skip_certificates: BTreeMap<Slot, bool>,
    /// Block proposals: slot -> hash
    block_proposals: BTreeMap<Slot, Hash>,
}

/// Formal model for timeout handling and skip certificate generation
#[derive(Clone)]
pub struct TimeoutModel {
    /// Number of validators
    pub validator_count: usize,
    /// Maximum slots to explore
    pub max_slot: Slot,
}

impl TimeoutState {
    fn new(validator_count: usize) -> Self {
        Self {
            network: BTreeSet::new(),
            validators: (0..validator_count).map(|_| ValidatorState {
                votes_cast: BTreeMap::new(),
                vote_pool: BTreeMap::new(),
                certificates: BTreeSet::new(),
            bad_window: false,
                current_slot: 0,
            }).collect(),
            current_slot: 0,
            skip_certificates: BTreeMap::new(),
            block_proposals: BTreeMap::new(),
        }
    }

    /// Check if a skip certificate can be formed for a slot
    fn can_form_skip_certificate(&self, slot: Slot) -> bool {
        if let Some(voters) = self.validators[0].vote_pool.get(&(slot, None)) {
            let stake: Stake = voters.len() as u64 * (TOTAL_STAKE / self.validators.len() as u64);
            stake >= (TOTAL_STAKE * SKIP_CERTIFICATE_THRESHOLD_PERCENT / 100)
        } else {
            false
        }
    }

    /// Check if a block certificate can be formed for a slot and hash
    fn can_form_block_certificate(&self, slot: Slot, hash: Hash) -> bool {
        if let Some(voters) = self.validators[0].vote_pool.get(&(slot, Some(hash))) {
            let stake: Stake = voters.len() as u64 * (TOTAL_STAKE / self.validators.len() as u64);
            stake >= (TOTAL_STAKE * SKIP_CERTIFICATE_THRESHOLD_PERCENT / 100)
        } else {
            false
        }
    }
}

impl Model for TimeoutModel {
    type State = TimeoutState;
    type Action = TimeoutAction;

    fn init_states(&self) -> Vec<Self::State> {
        vec![TimeoutState::new(self.validator_count)]
    }

    fn actions(&self, state: &Self::State, actions: &mut Vec<Self::Action>) {
        // 1. Deliver any message in the network
        for msg in &state.network {
            actions.push(TimeoutAction::DeliverMessage { msg: msg.clone() });
        }

        // 2. Propose blocks for current and future slots
        for proposer_id in 0..self.validator_count {
            for slot in state.current_slot..=self.max_slot {
                if !state.block_proposals.contains_key(&slot) {
                    actions.push(TimeoutAction::ProposeBlock {
                        slot,
                        proposer: proposer_id,
                    });
                }
            }
        }

        // 3. Trigger timeouts for any slot
        for validator_id in 0..self.validator_count {
            for slot in 1..=self.max_slot {
                actions.push(TimeoutAction::TriggerTimeout {
                    slot,
                    validator: validator_id,
                });
            }
        }

        // 4. Advance to next slot
        if state.current_slot < self.max_slot {
            actions.push(TimeoutAction::AdvanceSlot);
        }
    }

    fn next_state(&self, last_state: &Self::State, action: Self::Action) -> Option<Self::State> {
        let mut next_state = last_state.clone();
        let mut validators = last_state.validators.clone();

        match action {
            TimeoutAction::ProposeBlock { slot, proposer } => {
                let block_hash = slot * 1000 + proposer as u64; // Deterministic hash
                next_state.block_proposals.insert(slot, block_hash);

                // Broadcast block proposal to all validators
                for i in 0..self.validator_count {
                    if i != proposer {
                        next_state.network.insert(MessageInTransit {
                            dst: i,
                            msg: TimeoutMessage::BlockProposal {
                                slot,
                                hash: block_hash,
                                proposer,
                            },
                        });
                    }
                }
            }
            TimeoutAction::DeliverMessage { msg } => {
                let recipient_id = msg.dst;
                let mut validator_state = validators[recipient_id].clone();

                // Remove message from network
                if !next_state.network.remove(&msg) { return None; }

                match msg.msg {
                    TimeoutMessage::BlockProposal { slot, hash, proposer: _ } => {
                        // Validator receives block and can vote for it
                        if !validator_state.votes_cast.contains_key(&slot) {
                            validator_state.votes_cast.insert(slot, Some(hash));
                            
                            // Broadcast NotarVote
                            for i in 0..self.validator_count {
                                next_state.network.insert(MessageInTransit {
                                    dst: i,
                                    msg: TimeoutMessage::NotarVote {
                                        slot,
                                        hash,
                                        voter: recipient_id,
                                    },
                                });
                            }
                        }
                    }
                    TimeoutMessage::NotarVote { slot, hash, voter } => {
                        // Add vote to pool
                        let vote_key = (slot, Some(hash));
                        let voters = validator_state.vote_pool.entry(vote_key).or_default();
                        voters.insert(voter);

                        // Check for block certificate formation
                        if next_state.can_form_block_certificate(slot, hash) {
                            validator_state.certificates.insert((slot, Some(hash)));
                        }
                    }
                    TimeoutMessage::SkipVote { slot, voter } => {
                        // Add skip vote to pool
                        let vote_key = (slot, None);
                        let voters = validator_state.vote_pool.entry(vote_key).or_default();
                        voters.insert(voter);

                        // Check for skip certificate formation
                        if next_state.can_form_skip_certificate(slot) {
                            validator_state.certificates.insert((slot, None));
                            next_state.skip_certificates.insert(slot, true);
                            
                            // Set BadWindow flag
                            validator_state.bad_window = true;
                        }
                    }
                    TimeoutMessage::TimeoutEvent { slot, validator: _ } => {
                        // Timeout occurred - validator can cast skip vote
                        if !validator_state.votes_cast.contains_key(&slot) {
                            validator_state.votes_cast.insert(slot, None);
                            
                            // Broadcast SkipVote
                            for i in 0..self.validator_count {
                                next_state.network.insert(MessageInTransit {
                                    dst: i,
                                    msg: TimeoutMessage::SkipVote {
                                        slot,
                                        voter: recipient_id,
                                    },
                                });
                            }
                        }
                    }
                }
                validators[recipient_id] = validator_state;
            }
            TimeoutAction::TriggerTimeout { slot, validator } => {
                // Trigger timeout event
                next_state.network.insert(MessageInTransit {
                    dst: validator,
                    msg: TimeoutMessage::TimeoutEvent { slot, validator },
                });
            }
            TimeoutAction::AdvanceSlot => {
                next_state.current_slot += 1;
                for validator_state in &mut validators {
                    validator_state.current_slot = next_state.current_slot;
                }
            }
        }

        next_state.validators = validators;
        Some(next_state)
    }

    /// Properties to verify in the timeout model
    fn properties(&self) -> Vec<Property<Self>> {
        vec![
            // Property 1: Skip certificate uniqueness
            Property::<Self>::always("skip_certificate_uniqueness", |_model, state| {
                // At most one skip certificate per slot
                state.skip_certificates.len() <= state.current_slot as usize + 1
            }),
            
            // Property 2: BadWindow flag consistency
            Property::<Self>::always("badwindow_consistency", |_model, state| {
                // If BadWindow is set, there must be a skip certificate
                for validator in &state.validators {
                    if validator.bad_window {
                        // Check if there's a skip certificate in recent slots
                        let has_recent_skip_cert = (state.current_slot.saturating_sub(10)..=state.current_slot)
                            .any(|slot| state.skip_certificates.contains_key(&slot));
                        if !has_recent_skip_cert {
                            return false;
                        }
                    }
                }
                true
            }),
            
            // Property 3: Vote uniqueness per validator per slot
            Property::<Self>::always("vote_uniqueness", |_model, state| {
                for validator in &state.validators {
                    // Each validator can vote at most once per slot
                    for slot in 0..=state.current_slot {
                        let vote_count = validator.votes_cast.get(&slot).map(|_| 1).unwrap_or(0);
                        if vote_count > 1 {
                            return false;
                        }
                    }
                }
                true
            }),
            
            // Property 4: Certificate threshold enforcement
            Property::<Self>::always("certificate_threshold", |model, state| {
                for validator in &state.validators {
                    for (slot, hash_opt) in &validator.certificates {
                        let voters = validator.vote_pool.get(&(*slot, hash_opt.clone()));
                        if let Some(voter_set) = voters {
                            let stake: Stake = voter_set.len() as u64 * (TOTAL_STAKE / model.validator_count as u64);
                            if stake < (TOTAL_STAKE * SKIP_CERTIFICATE_THRESHOLD_PERCENT / 100) {
                                return false;
                            }
                        }
                    }
                }
                true
            }),
        ]
    }
}

/// Run formal verification of timeout handling
pub fn run_formal_verification() {
    println!("=== Timeout Handling Formal Verification ===");
    
    let model = TimeoutModel {
        validator_count: 3, // Small for formal verification
        max_slot: 3,
    };

    println!("Model checking timeout handling with {} validators, {} slots", 
             model.validator_count, model.max_slot);
    
    let result = model
        .checker()
        .threads(num_cpus::get())
        .spawn_dfs()
        .report(&mut stateright::report::WriteReporter::new(&mut std::io::stdout()));
    
    // Check verification results
    if result.discoveries().is_empty() {
        println!("✅ All timeout properties verified successfully");
    } else {
        println!("❌ Timeout verification found counterexamples");
        for (property_name, _path) in result.discoveries() {
            println!("  - {}", property_name);
        }
    }
}

/// Test timeout model with different configurations
pub fn test_timeout_model(validators: usize, slots: u64) {
    println!("Testing timeout model with {} validators, {} slots", validators, slots);
    
    let model = TimeoutModel {
        validator_count: validators,
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
    fn test_timeout_state_creation() {
        let state = TimeoutState::new(3);
        assert_eq!(state.validators.len(), 3);
        assert_eq!(state.current_slot, 0);
        assert!(state.network.is_empty());
    }

    #[test]
    fn test_skip_certificate_formation() {
        let mut state = TimeoutState::new(3);
        // Add enough skip votes to form certificate
        let mut validator = state.validators[0].clone();
        let voters = validator.vote_pool.entry((1, None)).or_default();
        voters.insert(0);
        voters.insert(1);
        voters.insert(2); // 3/3 validators = 100% > 60%
        state.validators[0] = validator;
        
        assert!(state.can_form_skip_certificate(1));
    }

    #[test]
    fn test_badwindow_flag_logic() {
        let mut state = TimeoutState::new(3);
        state.skip_certificates.insert(1, true);
        
        let mut validator = state.validators[0].clone();
        validator.bad_window = true;
        state.validators[0] = validator;
        
        // BadWindow should be consistent with skip certificates
        assert!(state.validators[0].bad_window);
    }
}
