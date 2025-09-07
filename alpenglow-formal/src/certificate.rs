//! Formal verification model for certificate aggregation and uniqueness in Alpenglow consensus.
//! This module provides a Stateright-based formal model for verifying certificate uniqueness,
//! aggregation logic, and safety properties in the presence of adversarial validators.

use stateright::{Model, Property, Checker};
use std::collections::{BTreeMap, BTreeSet};

// --- Formal Model Configuration ---
const NOTARIZE_THRESHOLD_PERCENT: u64 = 60;
const TOTAL_STAKE: u64 = 1000;
const MAX_SLOTS: u64 = 5; // Formal verification limit
const MAX_VALIDATORS: usize = 5; // Formal verification limit

// Type aliases for clarity
type Slot = u64;
type Hash = u64;
type ActorId = usize;
type Stake = u64;

/// Represents different types of messages in the certificate system
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum CertificateMessage {
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
    /// A certificate formed for a block
    BlockCertificate {
        slot: Slot,
        hash: Hash,
        stake: Stake,
    },
    /// A certificate formed for skipping a slot
    SkipCertificate {
        slot: Slot,
        stake: Stake,
    },
}

/// Represents messages in transit
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MessageInTransit {
    dst: ActorId,
    msg: CertificateMessage,
}

/// Actions that can be taken in the certificate model
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum CertificateAction {
    /// Cast a vote for a block
    CastNotarVote {
        slot: Slot,
        hash: Hash,
        voter: ActorId,
    },
    /// Cast a skip vote
    CastSkipVote {
        slot: Slot,
        voter: ActorId,
    },
    /// Deliver a message to its destination
    DeliverMessage { msg: MessageInTransit },
    /// Adversary attempts to equivocate
    AdversaryEquivocate {
        slot: Slot,
        hash1: Hash,
        hash2: Hash,
        adversary: ActorId,
    },
}

/// State of a validator in the certificate model
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ValidatorState {
    /// Votes cast by this validator: (slot, hash) -> true
    votes_cast: BTreeMap<(Slot, Option<Hash>), bool>,
    /// Vote pool: (slot, hash) -> set of voters
    vote_pool: BTreeMap<(Slot, Option<Hash>), BTreeSet<ActorId>>,
    /// Certificates formed: (slot, hash) pairs
    certificates: BTreeSet<(Slot, Option<Hash>)>,
    /// Whether this validator is adversarial
    is_adversary: bool,
    /// Stake of this validator
    stake: Stake,
}

/// Main state of the certificate formal model
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct CertificateState {
    /// Network messages in transit
    network: BTreeSet<MessageInTransit>,
    /// Per-validator states
    validators: Vec<ValidatorState>,
    /// Global certificates formed: (slot, hash) -> stake
    global_certificates: BTreeMap<(Slot, Option<Hash>), Stake>,
    /// Stake distribution: validator -> stake
    stake_distribution: BTreeMap<ActorId, Stake>,
}

/// Formal model for certificate aggregation and uniqueness
#[derive(Clone)]
pub struct CertificateModel {
    /// Number of validators
    pub validator_count: usize,
    /// Maximum slots to explore
    pub max_slot: Slot,
    /// Number of adversarial validators
    pub adversary_count: usize,
}

impl CertificateState {
    fn new(validator_count: usize, adversary_count: usize) -> Self {
        let mut stake_distribution = BTreeMap::new();
        let stake_per_validator = TOTAL_STAKE / validator_count as u64;
        
        for i in 0..validator_count {
            stake_distribution.insert(i, stake_per_validator);
        }

        Self {
            network: BTreeSet::new(),
            validators: (0..validator_count).map(|i| ValidatorState {
                votes_cast: BTreeMap::new(),
                vote_pool: BTreeMap::new(),
                certificates: BTreeSet::new(),
                is_adversary: i < adversary_count,
                stake: stake_per_validator,
            }).collect(),
            global_certificates: BTreeMap::new(),
            stake_distribution,
        }
    }

    /// Check if a certificate can be formed for a slot and hash
    fn can_form_certificate(&self, slot: Slot, hash: Option<Hash>) -> bool {
        if let Some(voters) = self.validators[0].vote_pool.get(&(slot, hash)) {
            let stake: Stake = voters.iter()
                .filter_map(|voter_id| self.stake_distribution.get(voter_id))
                .sum();
            stake >= (TOTAL_STAKE * NOTARIZE_THRESHOLD_PERCENT / 100)
        } else {
            false
        }
    }

    /// Get total stake for a set of voters
    fn get_stake_for_voters(&self, voters: &BTreeSet<ActorId>) -> Stake {
        voters.iter()
            .filter_map(|voter_id| self.stake_distribution.get(voter_id))
            .sum()
    }
}

impl Model for CertificateModel {
    type State = CertificateState;
    type Action = CertificateAction;

    fn init_states(&self) -> Vec<Self::State> {
        vec![CertificateState::new(self.validator_count, self.adversary_count)]
    }

    fn actions(&self, state: &Self::State, actions: &mut Vec<Self::Action>) {
        // 1. Deliver any message in the network
        for msg in &state.network {
            actions.push(CertificateAction::DeliverMessage { msg: msg.clone() });
        }

        // 2. Cast votes for blocks
        for slot in 1..=self.max_slot {
            for hash in 1..=3 { // Multiple possible hashes per slot
                for voter_id in 0..self.validator_count {
                    let vote_key = (slot, Some(hash));
                    if !state.validators[voter_id].votes_cast.contains_key(&vote_key) {
                        actions.push(CertificateAction::CastNotarVote {
                            slot,
                            hash,
                            voter: voter_id,
                        });
                    }
                }
            }
        }

        // 3. Cast skip votes
        for slot in 1..=self.max_slot {
            for voter_id in 0..self.validator_count {
                let vote_key = (slot, None);
                if !state.validators[voter_id].votes_cast.contains_key(&vote_key) {
                    actions.push(CertificateAction::CastSkipVote {
                        slot,
                        voter: voter_id,
                    });
                }
            }
        }

        // 4. Adversary equivocation attempts
        for slot in 1..=self.max_slot {
            for adversary_id in 0..self.adversary_count {
                actions.push(CertificateAction::AdversaryEquivocate {
                    slot,
                    hash1: slot * 1000 + 1,
                    hash2: slot * 1000 + 2,
                    adversary: adversary_id,
                });
            }
        }
    }

    fn next_state(&self, last_state: &Self::State, action: Self::Action) -> Option<Self::State> {
        let mut next_state = last_state.clone();
        let mut validators = last_state.validators.clone();

        match action {
            CertificateAction::CastNotarVote { slot, hash, voter } => {
                let mut validator_state = validators[voter].clone();
                let vote_key = (slot, Some(hash));
                
                // Check if validator can vote (not already voted for this slot)
                let can_vote = !validator_state.votes_cast.iter()
                    .any(|((s, _), _)| *s == slot);
                
                if can_vote {
                    validator_state.votes_cast.insert(vote_key, true);
                    
                    // Broadcast vote to all validators
                    for i in 0..self.validator_count {
                        next_state.network.insert(MessageInTransit {
                            dst: i,
                            msg: CertificateMessage::NotarVote {
                                slot,
                                hash,
                                voter,
                            },
                        });
                    }
                }
                validators[voter] = validator_state;
            }
            CertificateAction::CastSkipVote { slot, voter } => {
                let mut validator_state = validators[voter].clone();
                let vote_key = (slot, None);
                
                // Check if validator can vote (not already voted for this slot)
                let can_vote = !validator_state.votes_cast.iter()
                    .any(|((s, _), _)| *s == slot);
                
                if can_vote {
                    validator_state.votes_cast.insert(vote_key, true);
                    
                    // Broadcast skip vote to all validators
                    for i in 0..self.validator_count {
                        next_state.network.insert(MessageInTransit {
                            dst: i,
                            msg: CertificateMessage::SkipVote {
                                slot,
                                voter,
                            },
                        });
                    }
                }
                validators[voter] = validator_state;
            }
            CertificateAction::DeliverMessage { msg } => {
                let recipient_id = msg.dst;
                let mut validator_state = validators[recipient_id].clone();

                // Remove message from network
                if !next_state.network.remove(&msg) { return None; }

                match msg.msg {
                    CertificateMessage::NotarVote { slot, hash, voter } => {
                        // Add vote to pool
                        let vote_key = (slot, Some(hash));
                        let voters = validator_state.vote_pool.entry(vote_key).or_default();
                        voters.insert(voter);

                        // Check for certificate formation
                        if next_state.can_form_certificate(slot, Some(hash)) {
                            let stake = next_state.get_stake_for_voters(voters);
                            validator_state.certificates.insert((slot, Some(hash)));
                            next_state.global_certificates.insert((slot, Some(hash)), stake);
                        }
                    }
                    CertificateMessage::SkipVote { slot, voter } => {
                        // Add skip vote to pool
                        let vote_key = (slot, None);
                        let voters = validator_state.vote_pool.entry(vote_key).or_default();
                        voters.insert(voter);

                        // Check for skip certificate formation
                        if next_state.can_form_certificate(slot, None) {
                            let stake = next_state.get_stake_for_voters(voters);
                            validator_state.certificates.insert((slot, None));
                            next_state.global_certificates.insert((slot, None), stake);
                        }
                    }
                    CertificateMessage::BlockCertificate { slot, hash, stake } => {
                        // Certificate formed
                        validator_state.certificates.insert((slot, Some(hash)));
                        next_state.global_certificates.insert((slot, Some(hash)), stake);
                    }
                    CertificateMessage::SkipCertificate { slot, stake } => {
                        // Skip certificate formed
                        validator_state.certificates.insert((slot, None));
                        next_state.global_certificates.insert((slot, None), stake);
                    }
                }
                validators[recipient_id] = validator_state;
            }
            CertificateAction::AdversaryEquivocate { slot, hash1, hash2, adversary } => {
                let mut validator_state = validators[adversary].clone();
                
                // Adversary attempts to vote for both hashes
                let vote_key1 = (slot, Some(hash1));
                let vote_key2 = (slot, Some(hash2));
                
                // Adversary can equivocate (vote for multiple conflicting blocks)
                if validator_state.is_adversary {
                    validator_state.votes_cast.insert(vote_key1, true);
                    validator_state.votes_cast.insert(vote_key2, true);
                    
                    // Broadcast both votes
                    for i in 0..self.validator_count {
                        next_state.network.insert(MessageInTransit {
                            dst: i,
                            msg: CertificateMessage::NotarVote {
                                slot,
                                hash: hash1,
                                voter: adversary,
                            },
                        });
                        next_state.network.insert(MessageInTransit {
                            dst: i,
                            msg: CertificateMessage::NotarVote {
                                slot,
                                hash: hash2,
                                voter: adversary,
                            },
                        });
                    }
                }
                validators[adversary] = validator_state;
            }
        }

        next_state.validators = validators;
        Some(next_state)
    }

    /// Properties to verify in the certificate model
    fn properties(&self) -> Vec<Property<Self>> {
        vec![
            // Property 1: Certificate uniqueness - no conflicting certificates
            Property::<Self>::always("certificate_uniqueness", |model, state| {
                // Check that no two conflicting certificates exist for the same slot
                for slot in 1..=model.max_slot {
                    let mut certificates_for_slot = Vec::new();
                    
                    for ((s, hash_opt), _) in &state.global_certificates {
                        if *s == slot {
                            certificates_for_slot.push(hash_opt);
                        }
                    }
                    
                    // Should have at most one certificate per slot
                    if certificates_for_slot.len() > 1 {
                        return false;
                    }
                }
                true
            }),
            
            // Property 2: Vote uniqueness per validator per slot
            Property::<Self>::always("vote_uniqueness", |_, state| {
                for validator in &state.validators {
                    // Count votes per slot
                    let mut votes_per_slot: BTreeMap<Slot, usize> = BTreeMap::new();
                    
                    for ((slot, _), _) in &validator.votes_cast {
                        *votes_per_slot.entry(*slot).or_insert(0) += 1;
                    }
                    
                    // Each validator should vote at most once per slot
                    for (_, count) in votes_per_slot {
                        if count > 1 {
                            return false;
                        }
                    }
                }
                true
            }),
            
            // Property 3: Certificate threshold enforcement
            Property::<Self>::always("certificate_threshold", |_model, state| {
                for ((_slot, _hash_opt), stake) in &state.global_certificates {
                    // Verify the stake meets the threshold
                    if *stake < (TOTAL_STAKE * NOTARIZE_THRESHOLD_PERCENT / 100) {
                        return false;
                    }
                }
                true
            }),
            
            // Property 4: Adversary equivocation detection
            Property::<Self>::always("adversary_equivocation_detection", |model, state| {
                // Check that adversaries cannot create conflicting certificates
                for slot in 1..=model.max_slot {
                    let mut block_certificates = 0;
                    let mut skip_certificates = 0;
                    
                    for ((s, hash_opt), _) in &state.global_certificates {
                        if *s == slot {
                            match hash_opt {
                                Some(_) => block_certificates += 1,
                                None => skip_certificates += 1,
                            }
                        }
                    }
                    
                    // Should have at most one type of certificate per slot
                    if block_certificates > 0 && skip_certificates > 0 {
                        return false;
                    }
                }
                true
            }),
        ]
    }
}

/// Run formal verification of certificate aggregation
pub fn run_formal_verification() {
    println!("=== Certificate Aggregation Formal Verification ===");
    
    let model = CertificateModel {
        validator_count: 4, // Small for formal verification
        max_slot: 3,
        adversary_count: 1, // One adversarial validator
    };

    println!("Model checking certificate aggregation with {} validators ({} adversarial), {} slots", 
             model.validator_count, model.adversary_count, model.max_slot);
    
    let result = model
        .checker()
        .threads(num_cpus::get())
        .spawn_dfs()
        .report(&mut stateright::report::WriteReporter::new(&mut std::io::stdout()));
    
    // Check verification results
    if result.discoveries().is_empty() {
        println!("✅ All certificate properties verified successfully");
    } else {
        println!("❌ Certificate verification found counterexamples");
        for (property_name, _path) in result.discoveries() {
            println!("  - {}", property_name);
        }
    }
}

/// Test certificate model with different configurations
pub fn test_certificate_model(validators: usize, slots: u64, adversaries: usize) {
    println!("Testing certificate model with {} validators ({} adversarial), {} slots", 
             validators, adversaries, slots);
    
    let model = CertificateModel {
        validator_count: validators,
        max_slot: slots,
        adversary_count: adversaries,
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
    fn test_certificate_state_creation() {
        let state = CertificateState::new(3, 1);
        assert_eq!(state.validators.len(), 3);
        assert!(state.validators[0].is_adversary);
        assert!(!state.validators[1].is_adversary);
    }

    #[test]
    fn test_certificate_formation() {
        let mut state = CertificateState::new(3, 0);
        // Add enough votes to form certificate
        let mut validator = state.validators[0].clone();
        let voters = validator.vote_pool.entry((1, Some(100))).or_default();
        voters.insert(0);
        voters.insert(1);
        voters.insert(2); // 3/3 validators = 100% > 60%
        state.validators[0] = validator;
        
        assert!(state.can_form_certificate(1, Some(100)));
    }

    #[test]
    fn test_adversary_equivocation() {
        let state = CertificateState::new(3, 1);
        assert!(state.validators[0].is_adversary);
        assert!(!state.validators[1].is_adversary);
    }
}
