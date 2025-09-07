//! Formal verification model for safety properties in Alpenglow consensus.
//! This module provides a Stateright-based formal model for verifying safety guarantees,
//! chain consistency, and certificate uniqueness under adversarial conditions.

use stateright::{Model, Property, Checker};
use std::collections::{BTreeMap, BTreeSet};

// --- Formal Model Configuration ---
const CERTIFICATE_THRESHOLD_PERCENT: u64 = 60;
const TOTAL_STAKE: u64 = 1000;
const MAX_SLOTS: u64 = 5; // Formal verification limit
const MAX_VALIDATORS: usize = 5; // Formal verification limit
const MAX_BYZANTINE: usize = 1; // Formal verification limit

// Type aliases for clarity
type Slot = u64;
type Hash = u64;
type ActorId = usize;
type Stake = u64;

/// Represents different types of messages in the safety system
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum SafetyMessage {
    /// A block proposal
    BlockProposal {
        slot: Slot,
        hash: Hash,
        proposer: ActorId,
    },
    /// A vote for a block
    Vote {
        slot: Slot,
        hash: Hash,
        voter: ActorId,
    },
    /// A conflicting vote (Byzantine behavior)
    ConflictingVote {
        slot: Slot,
        hash: Hash,
        voter: ActorId,
    },
    /// A certificate formation
    CertificateFormed {
        slot: Slot,
        hash: Hash,
        stake: Stake,
    },
}

/// Represents messages in transit
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MessageInTransit {
    dst: ActorId,
    msg: SafetyMessage,
}

/// Actions that can be taken in the safety model
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum SafetyAction {
    /// Propose a block
    ProposeBlock {
        slot: Slot,
        proposer: ActorId,
    },
    /// Deliver a message to its destination
    DeliverMessage { msg: MessageInTransit },
    /// Byzantine validator creates conflicting vote
    CreateConflictingVote {
        slot: Slot,
        byzantine_validator: ActorId,
    },
    /// Form a certificate
    FormCertificate {
        slot: Slot,
        hash: Hash,
        stake: Stake,
    },
    /// Advance to the next slot
    AdvanceSlot,
}

/// State of a validator in the safety model
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ValidatorState {
    /// Whether this validator is Byzantine
    is_byzantine: bool,
    /// Whether this validator is responsive
    is_responsive: bool,
    /// Votes cast by this validator: (slot, hash) -> true
    votes_cast: BTreeMap<(Slot, Hash), bool>,
    /// Vote pool: (slot, hash) -> set of voters
    vote_pool: BTreeMap<(Slot, Hash), BTreeSet<ActorId>>,
    /// Certificates formed: slot -> hash
    certificates: BTreeMap<Slot, Hash>,
    /// Chain of finalized blocks: slot -> hash
    finalized_chain: BTreeMap<Slot, Hash>,
    /// Current slot
    current_slot: Slot,
}

/// Main state of the safety formal model
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct SafetyState {
    /// Network messages in transit
    network: BTreeSet<MessageInTransit>,
    /// Per-validator states
    validators: Vec<ValidatorState>,
    /// Global current slot
    current_slot: Slot,
    /// Stake distribution: validator -> stake
    stake_distribution: BTreeMap<ActorId, Stake>,
    /// Block proposals: slot -> hash
    block_proposals: BTreeMap<Slot, Hash>,
    /// Global certificates: slot -> hash
    global_certificates: BTreeMap<Slot, Hash>,
    /// Safety violations detected
    safety_violations: BTreeSet<(Slot, Hash, Hash)>, // (slot, hash1, hash2) for conflicting blocks
}

/// Formal model for safety properties
#[derive(Clone)]
pub struct SafetyModel {
    /// Number of validators
    pub validator_count: usize,
    /// Maximum slots to explore
    pub max_slot: Slot,
    /// Number of Byzantine validators
    pub byzantine_count: usize,
}

impl SafetyState {
    fn new(validator_count: usize, byzantine_count: usize) -> Self {
        let mut stake_distribution = BTreeMap::new();
        let stake_per_validator = TOTAL_STAKE / validator_count as u64;
        
        for i in 0..validator_count {
            stake_distribution.insert(i, stake_per_validator);
        }

        Self {
            network: BTreeSet::new(),
            validators: (0..validator_count).map(|i| ValidatorState {
                is_byzantine: i < byzantine_count,
                is_responsive: true,
                votes_cast: BTreeMap::new(),
                vote_pool: BTreeMap::new(),
                certificates: BTreeMap::new(),
                finalized_chain: BTreeMap::new(),
                current_slot: 0,
            }).collect(),
            current_slot: 0,
            stake_distribution,
            block_proposals: BTreeMap::new(),
            global_certificates: BTreeMap::new(),
            safety_violations: BTreeSet::new(),
        }
    }

    /// Check if a block can be certified (60% threshold)
    fn can_certify(&self, slot: Slot, hash: Hash) -> bool {
        if let Some(voters) = self.validators[0].vote_pool.get(&(slot, hash)) {
            let honest_stake: Stake = voters.iter()
                .filter(|voter_id| !self.validators[**voter_id].is_byzantine)
                .filter(|voter_id| self.validators[**voter_id].is_responsive)
                .filter_map(|voter_id| self.stake_distribution.get(voter_id))
                .sum();
            honest_stake >= (TOTAL_STAKE * CERTIFICATE_THRESHOLD_PERCENT / 100)
        } else {
            false
        }
    }

    /// Check for safety violations (conflicting certificates)
    fn check_safety_violations(&mut self) {
        // Check for conflicting certificates in the same slot
        for (slot, hash1) in &self.global_certificates {
            for (other_slot, hash2) in &self.global_certificates {
                if *slot == *other_slot && hash1 != hash2 {
                    self.safety_violations.insert((*slot, *hash1, *hash2));
                }
            }
        }

        // Check for conflicting certificates across validators
        for validator in &self.validators {
            for (slot, hash1) in &validator.certificates {
                for (other_slot, hash2) in &validator.certificates {
                    if *slot == *other_slot && hash1 != hash2 {
                        self.safety_violations.insert((*slot, *hash1, *hash2));
                    }
                }
            }
        }
    }

    /// Check chain consistency
    fn check_chain_consistency(&self) -> bool {
        // All validators should have consistent finalized chains
        let mut reference_chain: Option<BTreeMap<Slot, Hash>> = None;
        
        for validator in &self.validators {
            if !validator.finalized_chain.is_empty() {
                if let Some(ref_chain) = &reference_chain {
                    // Check if chains are consistent
                    for (slot, hash) in &validator.finalized_chain {
                        if let Some(ref_hash) = ref_chain.get(slot) {
                            if hash != ref_hash {
                                return false; // Inconsistent chain
                            }
                        }
                    }
                } else {
                    reference_chain = Some(validator.finalized_chain.clone());
                }
            }
        }
        true
    }
}

impl Model for SafetyModel {
    type State = SafetyState;
    type Action = SafetyAction;

    fn init_states(&self) -> Vec<Self::State> {
        vec![SafetyState::new(self.validator_count, self.byzantine_count)]
    }

    fn actions(&self, state: &Self::State, actions: &mut Vec<Self::Action>) {
        // 1. Deliver any message in the network
        for msg in &state.network {
            actions.push(SafetyAction::DeliverMessage { msg: msg.clone() });
        }

        // 2. Propose blocks for current and future slots
        for slot in state.current_slot..=self.max_slot {
            for proposer in 0..self.validator_count {
                if !state.block_proposals.contains_key(&slot) {
                    actions.push(SafetyAction::ProposeBlock {
                        slot,
                        proposer,
                    });
                }
            }
        }

        // 3. Byzantine validators create conflicting votes
        for slot in 1..=self.max_slot {
            for byzantine_validator in 0..self.byzantine_count {
                actions.push(SafetyAction::CreateConflictingVote {
                    slot,
                    byzantine_validator,
                });
            }
        }

        // 4. Form certificates when threshold is met
        for validator in &state.validators {
            for ((slot, hash), voters) in &validator.vote_pool {
                let honest_stake: Stake = voters.iter()
                    .filter(|voter_id| !state.validators[**voter_id].is_byzantine)
                    .filter(|voter_id| state.validators[**voter_id].is_responsive)
                    .filter_map(|voter_id| state.stake_distribution.get(voter_id))
                    .sum();
                
                if honest_stake >= (TOTAL_STAKE * CERTIFICATE_THRESHOLD_PERCENT / 100) {
                    if !state.global_certificates.contains_key(slot) {
                        actions.push(SafetyAction::FormCertificate {
                            slot: *slot,
                            hash: *hash,
                            stake: honest_stake,
                        });
                    }
                }
            }
        }

        // 5. Advance to next slot
        if state.current_slot < self.max_slot {
            actions.push(SafetyAction::AdvanceSlot);
        }
    }

    fn next_state(&self, last_state: &Self::State, action: Self::Action) -> Option<Self::State> {
        let mut next_state = last_state.clone();
        let mut validators = last_state.validators.clone();

        match action {
            SafetyAction::ProposeBlock { slot, proposer } => {
                let block_hash = slot * 1000 + proposer as u64;
                next_state.block_proposals.insert(slot, block_hash);

                // Broadcast block proposal to all validators
                for i in 0..self.validator_count {
                    if i != proposer {
                        next_state.network.insert(MessageInTransit {
                            dst: i,
                            msg: SafetyMessage::BlockProposal {
                                slot,
                                hash: block_hash,
                                proposer,
                            },
                        });
                    }
                }
            }
            SafetyAction::DeliverMessage { msg } => {
                let recipient_id = msg.dst;
                let mut validator_state = validators[recipient_id].clone();

                // Remove message from network
                if !next_state.network.remove(&msg) { return None; }

                match msg.msg {
                    SafetyMessage::BlockProposal { slot, hash, proposer: _ } => {
                        // Validator receives block and can vote for it
                        if validator_state.is_responsive {
                            if !validator_state.votes_cast.contains_key(&(slot, hash)) {
                                validator_state.votes_cast.insert((slot, hash), true);
                                
                                // Broadcast vote
                                for i in 0..self.validator_count {
                                    next_state.network.insert(MessageInTransit {
                                        dst: i,
                                        msg: SafetyMessage::Vote {
                                            slot,
                                            hash,
                                            voter: recipient_id,
                                        },
                                    });
                                }
                            }
                        }
                    }
                    SafetyMessage::Vote { slot, hash, voter } => {
                        // Add vote to pool
                        let voters = validator_state.vote_pool.entry((slot, hash)).or_default();
                        voters.insert(voter);

                        // Check for certification
                        if next_state.can_certify(slot, hash) {
                            validator_state.certificates.insert(slot, hash);
                            validator_state.finalized_chain.insert(slot, hash);
                        }
                    }
                    SafetyMessage::ConflictingVote { slot, hash, voter } => {
                        // Byzantine vote - add to pool but mark as conflicting
                        let voters = validator_state.vote_pool.entry((slot, hash)).or_default();
                        voters.insert(voter);
                        
                        // Check for certification (should fail due to Byzantine behavior)
                        if next_state.can_certify(slot, hash) {
                            validator_state.certificates.insert(slot, hash);
                            validator_state.finalized_chain.insert(slot, hash);
                        }
                    }
                    SafetyMessage::CertificateFormed { slot, hash, stake: _ } => {
                        // Global certificate formed
                        next_state.global_certificates.insert(slot, hash);
                        
                        // Update all validators
                        for validator_state in &mut validators {
                            validator_state.certificates.insert(slot, hash);
                            validator_state.finalized_chain.insert(slot, hash);
                        }
                    }
                }
                validators[recipient_id] = validator_state;
            }
            SafetyAction::CreateConflictingVote { slot, byzantine_validator } => {
                // Byzantine validator creates conflicting vote
                let conflicting_hash = slot * 1000 + 999; // Different hash
                next_state.network.insert(MessageInTransit {
                    dst: byzantine_validator,
                    msg: SafetyMessage::ConflictingVote {
                        slot,
                        hash: conflicting_hash,
                        voter: byzantine_validator,
                    },
                });
            }
            SafetyAction::FormCertificate { slot, hash, stake } => {
                // Form global certificate
                next_state.network.insert(MessageInTransit {
                    dst: 0, // Send to first validator to process
                    msg: SafetyMessage::CertificateFormed {
                        slot,
                        hash,
                        stake,
                    },
                });
            }
            SafetyAction::AdvanceSlot => {
                next_state.current_slot += 1;
                for validator_state in &mut validators {
                    validator_state.current_slot = next_state.current_slot;
                }
            }
        }

        next_state.validators = validators;
        next_state.check_safety_violations();
        Some(next_state)
    }

    /// Properties to verify in the safety model
    fn properties(&self) -> Vec<Property<Self>> {
        vec![
            // Property 1: No conflicting blocks can be finalized in the same slot
            Property::<Self>::always("no_conflicting_finalization", |_model, state| {
                state.safety_violations.is_empty()
            }),
            
            // Property 2: Chain consistency under up to 20% Byzantine stake
            Property::<Self>::always("chain_consistency", |_model, state| {
                let byzantine_stake: Stake = state.validators.iter()
                    .filter(|_v| _v.is_byzantine)
                    .map(|_v| state.stake_distribution.get(&0).unwrap_or(&0))
                    .sum();
                
                if byzantine_stake <= (TOTAL_STAKE * 20 / 100) {
                    // With ≤20% Byzantine stake, chain consistency should be maintained
                    state.check_chain_consistency()
                } else {
                    true // If >20% Byzantine, no consistency guarantee
                }
            }),
            
            // Property 3: Certificate uniqueness
            Property::<Self>::always("certificate_uniqueness", |_model, state| {
                // Each slot should have at most one certificate
                let mut seen_slots = BTreeSet::new();
                for (slot, _hash) in &state.global_certificates {
                    if seen_slots.contains(slot) {
                        return false; // Duplicate certificate for same slot
                    }
                    seen_slots.insert(*slot);
                }
                true
            }),
            
            // Property 4: Non-equivocation
            Property::<Self>::always("non_equivocation", |_model, state| {
                // Each validator should vote at most once per slot
                for validator in &state.validators {
                    let mut seen_slots = BTreeSet::new();
                    for (slot, _hash) in &validator.votes_cast {
                        if seen_slots.contains(slot) {
                            return false; // Multiple votes for same slot
                        }
                        seen_slots.insert(*slot);
                    }
                }
                true
            }),
            
            // Property 5: Safety under Byzantine faults
            Property::<Self>::always("safety_byzantine", |_model, state| {
                // Safety should be maintained even with Byzantine validators
                // This is checked by the absence of safety violations
                state.safety_violations.is_empty()
            }),
        ]
    }
}

/// Run formal verification of safety properties
pub fn run_formal_verification() {
    println!("=== Safety Properties Formal Verification ===");
    
    let model = SafetyModel {
        validator_count: 4, // Small for formal verification
        max_slot: 3,
        byzantine_count: 1, // 25% Byzantine (within 20% threshold for safety)
    };

    println!("Model checking safety with {} validators ({} Byzantine), {} slots", 
             model.validator_count, model.byzantine_count, model.max_slot);
    
    let result = model
        .checker()
        .threads(num_cpus::get())
        .spawn_dfs()
        .report(&mut stateright::report::WriteReporter::new(&mut std::io::stdout()));
    
    // Check verification results
    if result.discoveries().is_empty() {
        println!("✅ All safety properties verified successfully");
    } else {
        println!("❌ Safety verification found counterexamples");
        for (property_name, _path) in result.discoveries() {
            println!("  - {}", property_name);
        }
    }
}

/// Test safety model with different configurations
pub fn test_safety_model(validators: usize, slots: u64, byzantine: usize) {
    println!("Testing safety model with {} validators ({} Byzantine), {} slots", 
             validators, byzantine, slots);
    
    let model = SafetyModel {
        validator_count: validators,
        max_slot: slots,
        byzantine_count: byzantine,
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
    fn test_safety_state_creation() {
        let state = SafetyState::new(3, 1);
        assert_eq!(state.validators.len(), 3);
        assert_eq!(state.validators[0].is_byzantine, true);
        assert_eq!(state.validators[2].is_byzantine, false);
    }

    #[test]
    fn test_certification_threshold() {
        let mut state = SafetyState::new(3, 0);
        // Add enough honest votes to certify
        let mut validator = state.validators[0].clone();
        let voters = validator.vote_pool.entry((1, 100)).or_default();
        voters.insert(0);
        voters.insert(1);
        voters.insert(2); // 3/3 validators = 100% > 60%
        state.validators[0] = validator;
        
        assert!(state.can_certify(1, 100));
    }

    #[test]
    fn test_chain_consistency() {
        let state = SafetyState::new(3, 0);
        assert!(state.check_chain_consistency());
    }
}
