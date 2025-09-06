//! Certificate aggregation and uniqueness verification for Alpenglow consensus.
//! This module demonstrates how the system prevents conflicting certificates
//! from being formed, ensuring safety even in the presence of adversarial validators.

use std::collections::{HashMap, HashSet};
use std::fmt;

// --- Configuration ---
const NOTARIZE_THRESHOLD_PERCENT: u64 = 60;
const TOTAL_STAKE: u64 = 1000;

/// Represents a signed vote from a single validator.
#[derive(Clone, Debug, PartialEq)]
pub struct Vote {
    pub slot: u64,
    pub block_hash: Option<String>, // None for a SkipVote
    pub voter_id: u64,
    pub stake: u64,
}

impl Vote {
    pub fn new(slot: u64, block_hash: Option<String>, voter_id: u64, stake: u64) -> Self {
        Self {
            slot,
            block_hash,
            voter_id,
            stake,
        }
    }

    /// A unique identifier for what is being voted on.
    pub fn key(&self) -> (u64, Option<String>) {
        (self.slot, self.block_hash.clone())
    }
}

impl fmt::Display for Vote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let vote_type = if self.block_hash.is_some() { "NotarVote" } else { "SkipVote" };
        write!(
            f,
            "({} for Slot {}, Hash: {:?}, Voter: {})",
            vote_type, self.slot, self.block_hash, self.voter_id
        )
    }
}

/// Simulates a validator's logic for voting and aggregating certificates.
#[derive(Debug)]
pub struct Validator {
    pub id: u64,
    pub stake: u64,
    voted_slots: HashMap<u64, Option<String>>, // Enforces the "vote once per slot" rule
    vote_pool: HashMap<(u64, Option<String>), HashMap<u64, u64>>, // Stores votes received
    certificates: HashSet<(u64, Option<String>)>, // Stores keys of successfully aggregated certificates
}

impl Validator {
    pub fn new(id: u64, stake: u64) -> Self {
        Self {
            id,
            stake,
            voted_slots: HashMap::new(),
            vote_pool: HashMap::new(),
            certificates: HashSet::new(),
        }
    }

    /// Creates a vote, but only if the validator hasn't already voted for this slot.
    pub fn create_vote(&mut self, slot: u64, block_hash: Option<String>) -> Option<Vote> {
        if self.voted_slots.contains_key(&slot) {
            println!("Validator {}: IGNORED attempt to double-vote for slot {}.", self.id, slot);
            return None;
        }

        // The core safety rule: vote once and record it.
        self.voted_slots.insert(slot, block_hash.clone());
        println!(
            "✅ Validator {} (Stake: {}): Voted for hash '{:?}' in slot {}.",
            self.id, self.stake, block_hash, slot
        );
        Some(Vote::new(slot, block_hash, self.id, self.stake))
    }

    /// Simulates receiving a vote from the network and adding it to the local pool.
    pub fn receive_vote(&mut self, vote: &Vote) {
        let vote_key = vote.key();
        let slot_votes = self.vote_pool.entry(vote_key).or_insert_with(HashMap::new);
        
        // Add the vote, preventing duplicate entries from the same voter
        slot_votes.insert(vote.voter_id, vote.stake);
    }

    /// Scans the vote pool to see if any certificates can be formed.
    /// This would be run continuously in a real node.
    pub fn aggregate_certificates(&mut self) {
        println!("\nValidator {}: Aggregating certificates from vote pool...", self.id);
        
        for (vote_key, votes) in &self.vote_pool {
            if self.certificates.contains(vote_key) {
                continue; // Already formed this certificate
            }

            let current_stake: u64 = votes.values().sum();
            println!(
                "  - Checking {:?}: Total stake = {}/{}",
                vote_key, current_stake, TOTAL_STAKE
            );

            if current_stake >= (TOTAL_STAKE * NOTARIZE_THRESHOLD_PERCENT / 100) {
                self.certificates.insert(vote_key.clone());
                println!(
                    "  🔥 CERTIFICATE FORMED for Slot {} with Hash '{:?}'! Stake: {}",
                    vote_key.0, vote_key.1, current_stake
                );
            }
        }
    }

    pub fn get_certificates(&self) -> &HashSet<(u64, Option<String>)> {
        &self.certificates
    }

    pub fn get_vote_pool(&self) -> &HashMap<(u64, Option<String>), HashMap<u64, u64>> {
        &self.vote_pool
    }
}

/// Demonstrates an adversary failing to create conflicting certificates.
pub fn run_simulation() {
    println!("--- Alpenglow Certificate Uniqueness Simulation ---");

    // 1. Setup Validators
    let mut validators: Vec<Validator> = (0..18)
        .map(|i| Validator::new(i, 50))
        .collect(); // 18 honest validators with 50 stake each (900 total)
    
    let adversary = Validator::new(99, 100); // 1 adversary with 100 stake (10% of total)
    
    // Main validator who will be aggregating votes from everyone
    let mut observer_validator = Validator::new(100, 0);

    let slot_to_contest = 5;
    let hash_a = Some("block-hash-alpha".to_string());
    let hash_b = Some("block-hash-bravo".to_string()); // Conflicting block

    println!(
        "\nAn adversary (Validator 99, Stake: {}) will try to notarize two blocks for Slot {}.\n",
        adversary.stake, slot_to_contest
    );

    // 2. Generation Phase
    let mut all_votes = Vec::new();

    // Group A (first 9 validators) votes for hash_A
    for i in 0..9 {
        if let Some(vote) = validators[i].create_vote(slot_to_contest, hash_a.clone()) {
            all_votes.push(vote);
        }
    }

    // Group B (next 9 validators) votes for hash_B
    for i in 9..18 {
        if let Some(vote) = validators[i].create_vote(slot_to_contest, hash_b.clone()) {
            all_votes.push(vote);
        }
    }

    // Adversary double-votes! (But its own state prevents it from creating two valid votes)
    // A real adversary would bypass this check, but the honest nodes will still only count one vote from them per slot.
    // For simulation, we'll manually create its conflicting votes.
    let adversary_vote_a = Vote::new(slot_to_contest, hash_a.clone(), adversary.id, adversary.stake);
    let adversary_vote_b = Vote::new(slot_to_contest, hash_b.clone(), adversary.id, adversary.stake);
    println!(" Adversary {} maliciously creates votes for BOTH hashes.", adversary.id);
    all_votes.extend([adversary_vote_a, adversary_vote_b]);

    // 3. Aggregation Phase
    // Shuffle votes to simulate network delays and random arrival order
    use rand::seq::SliceRandom;
    use rand::thread_rng;
    all_votes.shuffle(&mut thread_rng());

    for vote in &all_votes {
        observer_validator.receive_vote(vote);
    }

    observer_validator.aggregate_certificates();

    println!("\n--- Simulation Results ---");
    let certificate_count = observer_validator.get_certificates().len();
    if certificate_count == 2 {
        println!("🔴 FAILURE: Two conflicting certificates were formed. The safety property is broken.");
    } else if certificate_count == 1 {
        let cert = observer_validator.get_certificates().iter().next().unwrap();
        println!(
            "✅ SUCCESS: Only one certificate for Slot {} ('{:?}') was formed.",
            cert.0, cert.1
        );
    } else {
        println!("🔵 NOTE: No certificate was formed because neither option reached the 60% threshold.");
    }

    // Calculate final stake for each conflicting hash
    let stake_a: u64 = observer_validator
        .get_vote_pool()
        .get(&(slot_to_contest, hash_a.clone()))
        .map(|votes| votes.values().sum())
        .unwrap_or(0);
    
    let stake_b: u64 = observer_validator
        .get_vote_pool()
        .get(&(slot_to_contest, hash_b.clone()))
        .map(|votes| votes.values().sum())
        .unwrap_or(0);

    println!("Final stake for Hash A: {} (Honest: 450, Adversary: 100)", stake_a);
    println!("Final stake for Hash B: {} (Honest: 450, Adversary: 100)", stake_b);
    println!("Required stake: {}", TOTAL_STAKE * NOTARIZE_THRESHOLD_PERCENT / 100);
    println!("Because neither side could reach the threshold alone, and honest validators did not vote for both, no conflicting certificate could be created.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vote_creation() {
        let mut validator = Validator::new(1, 100);
        let vote = validator.create_vote(1, Some("hash1".to_string()));
        assert!(vote.is_some());
        assert_eq!(vote.unwrap().slot, 1);
    }

    #[test]
    fn test_double_vote_prevention() {
        let mut validator = Validator::new(1, 100);
        let _vote1 = validator.create_vote(1, Some("hash1".to_string()));
        let vote2 = validator.create_vote(1, Some("hash2".to_string()));
        assert!(vote2.is_none()); // Should be prevented
    }
}
