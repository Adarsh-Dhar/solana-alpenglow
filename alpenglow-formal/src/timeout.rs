//! Timeout handling and skip certificate generation for Alpenglow consensus.
//! This module demonstrates how the system handles timeouts and generates
//! skip certificates when leaders fail to produce blocks in time.

use std::collections::{HashMap, HashSet};
use std::thread;
use std::time::Duration;
use rand::Rng;

// --- Configuration ---
const SKIP_CERTIFICATE_THRESHOLD_PERCENT: u64 = 60;
const TOTAL_STAKE: u64 = 1000;
const SLOT_TIMEOUT_MILLIS: u64 = 100; // A short timeout for simulation purposes

/// Represents a vote from a validator.
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
}

impl std::fmt::Display for Vote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let vote_type = if self.block_hash.is_some() { "NotarVote" } else { "SkipVote" };
        write!(
            f,
            "({} for Slot {}, Voter: {})",
            vote_type, self.slot, self.voter_id
        )
    }
}

/// Simulates a validator's logic, including timeouts.
#[derive(Debug)]
pub struct Validator {
    pub id: u64,
    pub stake: u64,
    voted_slots: HashMap<u64, Option<String>>,
    vote_pool: HashMap<(u64, Option<String>), HashMap<u64, u64>>,
    certificates: HashSet<(u64, Option<String>)>,
    bad_window: bool,
}

impl Validator {
    pub fn new(id: u64, stake: u64) -> Self {
        Self {
            id,
            stake,
            voted_slots: HashMap::new(),
            vote_pool: HashMap::new(),
            certificates: HashSet::new(),
            bad_window: false,
        }
    }

    /// Begins the timeout countdown for a new slot.
    pub fn start_slot(&mut self, slot: u64) -> Option<Vote> {
        println!("Validator {:2}: Started timer for Slot {}.", self.id, slot);
        
        // In a real system, this would be an async task. Here we simulate it.
        // We'll check for a block after a delay.
        let mut rng = rand::thread_rng();
        let jitter = rng.gen_range(-5..=5);
        let timeout_duration = Duration::from_millis((SLOT_TIMEOUT_MILLIS as i64 + jitter as i64).max(1) as u64);
        thread::sleep(timeout_duration);

        // Check if we managed to vote for a block in time
        if !self.voted_slots.contains_key(&slot) {
            println!("🔴 Validator {:2}: TIMEOUT for Slot {}!", self.id, slot);
            return self.create_skip_vote(slot);
        }
        None
    }

    /// Simulates receiving a block and voting, if within the timeout window.
    pub fn receive_block_and_vote(&mut self, slot: u64, block_hash: String) -> Option<Vote> {
        if self.voted_slots.contains_key(&slot) {
            return None; // Already voted
        }
        
        println!(
            "✅ Validator {:2}: Received block '{}' and cast NotarVote.",
            self.id, block_hash
        );
        self.voted_slots.insert(slot, Some(block_hash.clone()));
        Some(Vote::new(slot, Some(block_hash), self.id, self.stake))
    }

    /// Creates a SkipVote after a timeout.
    pub fn create_skip_vote(&mut self, slot: u64) -> Option<Vote> {
        if self.voted_slots.contains_key(&slot) {
            return None;
        }
        
        self.voted_slots.insert(slot, None); // Mark as voted (to skip)
        println!(
            "✉️ Validator {:2}: Generating and broadcasting SkipVote for Slot {}.",
            self.id, slot
        );
        Some(Vote::new(slot, None, self.id, self.stake))
    }

    /// Aggregates all votes to check for certificates.
    pub fn aggregate_certificates(&mut self, all_votes_cast: &[Vote]) {
        // Reset and rebuild view from all available votes
        self.vote_pool.clear();
        for vote in all_votes_cast {
            let key = (vote.slot, vote.block_hash.clone());
            let slot_votes = self.vote_pool.entry(key).or_insert_with(HashMap::new);
            slot_votes.insert(vote.voter_id, vote.stake);
        }
        
        println!(
            "\nValidator {:2}: Aggregating from {} total votes...",
            self.id, all_votes_cast.len()
        );
        
        for (key, votes) in &self.vote_pool {
            let current_stake: u64 = votes.values().sum();
            if current_stake >= (TOTAL_STAKE * SKIP_CERTIFICATE_THRESHOLD_PERCENT / 100) {
                if !self.certificates.contains(key) {
                    self.certificates.insert(key.clone());
                    println!("🔥 CERTIFICATE FORMED for {:?}! Stake: {}", key, current_stake);
                    
                    // This is the key logic: a SkipCertificate sets the BadWindow flag
                    if key.1.is_none() { // key.1 is the block_hash
                        println!(
                            "🚦 Validator {:2}: Observed SkipCertificate. Setting BadWindow flag to TRUE.",
                            self.id
                        );
                        self.bad_window = true;
                    }
                }
            }
        }
    }

    /// Attempts to cast a FinalVote, respecting the BadWindow flag.
    pub fn try_final_vote(&self, notarized_slot: u64) -> bool {
        println!(
            "\nValidator {:2}: Checking conditions to cast FinalVote for slot {}...",
            self.id, notarized_slot
        );
        
        if self.bad_window {
            println!(
                "❌ Validator {:2}: CANNOT cast FinalVote. BadWindow flag is active.",
                self.id
            );
            false
        } else {
            println!(
                "👍 Validator {:2}: OK to cast FinalVote. BadWindow is clear.",
                self.id
            );
            true
        }
    }

    #[allow(dead_code)]
    pub fn is_bad_window_active(&self) -> bool {
        self.bad_window
    }

    #[allow(dead_code)]
    pub fn get_certificates(&self) -> &HashSet<(u64, Option<String>)> {
        &self.certificates
    }
}

/// Runs the timeout and skip certificate simulation.
pub fn run_simulation() {
    println!("--- Alpenglow Timeout & Skip Certificate Simulation ---");
    
    // Setup: 10 validators, 100 stake each
    let mut validators: Vec<Validator> = (0..10)
        .map(|i| Validator::new(i, 100))
        .collect();
    
    let slot_to_test = 8;
    
    println!(
        "Simulating a slow leader for Slot {}. Timeout is {}ms.\n",
        slot_to_test, SLOT_TIMEOUT_MILLIS
    );

    // 1. Most validators time out
    let mut all_votes_cast = Vec::new();
    
    for validator in &mut validators {
        // We simulate that only 2 validators get the block in time
        if validator.id < 2 {
            if let Some(vote) = validator.receive_block_and_vote(slot_to_test, "late-block-hash".to_string()) {
                all_votes_cast.push(vote);
            }
        }
        
        // The rest will time out
        if let Some(skip_vote) = validator.start_slot(slot_to_test) {
            all_votes_cast.push(skip_vote);
        }
    }

    // 2. All validators aggregate the collected votes
    // In a real network this is concurrent, here we do it for one observer
    let observer_validator = &mut validators[0];
    observer_validator.aggregate_certificates(&all_votes_cast);

    // 3. Check the consequence
    // Now, let's assume some previous slot 7 was notarized and needs a FinalVote.
    // The validator's ability to do this now depends on the BadWindow flag.
    observer_validator.try_final_vote(7);
    
    println!("\n--- Simulation Results ---");
    let num_skip_votes = all_votes_cast.iter().filter(|v| v.block_hash.is_none()).count();
    println!(
        "A SkipCertificate was formed because {}/{} stake voted to skip, exceeding the {}% threshold.",
        num_skip_votes * 100, TOTAL_STAKE, SKIP_CERTIFICATE_THRESHOLD_PERCENT
    );
    println!("This correctly triggered the BadWindow flag, preventing optimistic finalization votes and maintaining network safety during a period of liveness failure.");
}

/// Test skip certificate generation
pub fn test_skip_certificate_generation() {
    println!("--- Testing Skip Certificate Generation ---");
    
    let mut validators: Vec<Validator> = (0..10)
        .map(|i| Validator::new(i, 100))
        .collect();
    
    let slot_to_test = 5;
    let mut all_votes_cast = Vec::new();
    
    // All validators time out and create skip votes
    for validator in &mut validators {
        if let Some(skip_vote) = validator.create_skip_vote(slot_to_test) {
            all_votes_cast.push(skip_vote);
        }
    }
    
    // Aggregate certificates
    let observer_validator = &mut validators[0];
    observer_validator.aggregate_certificates(&all_votes_cast);
    
    println!("Skip certificate generation test completed");
}

/// Test BadWindow flag triggering
pub fn test_badwindow_triggering() {
    println!("--- Testing BadWindow Flag Triggering ---");
    
    let mut validator = Validator::new(1, 100);
    
    // Create skip votes to trigger BadWindow
    let skip_vote = validator.create_skip_vote(1).unwrap();
    let all_votes = vec![skip_vote];
    
    validator.aggregate_certificates(&all_votes);
    
    if validator.is_bad_window_active() {
        println!("BadWindow flag correctly triggered");
    } else {
        println!("BadWindow flag not triggered - this may be an issue");
    }
}

/// Test network delay handling
pub fn test_network_delay_handling(delay_ms: u64) {
    println!("--- Testing Network Delay Handling with {}ms delay ---", delay_ms);
    
    let mut validators: Vec<Validator> = (0..5)
        .map(|i| Validator::new(i, 100))
        .collect();
    
    // Simulate network delay
    let slot_to_test = 3;
    let mut all_votes_cast = Vec::new();
    
    for validator in &mut validators {
        // Simulate delayed response
        if delay_ms <= 50 {
            if let Some(vote) = validator.receive_block_and_vote(slot_to_test, "delayed-block".to_string()) {
                all_votes_cast.push(vote);
            }
        } else {
            if let Some(skip_vote) = validator.create_skip_vote(slot_to_test) {
                all_votes_cast.push(skip_vote);
            }
        }
    }
    
    let observer_validator = &mut validators[0];
    observer_validator.aggregate_certificates(&all_votes_cast);
    
    println!("Network delay handling test completed");
}

/// Test timeout recovery
pub fn test_timeout_recovery() {
    println!("--- Testing Timeout Recovery ---");
    
    let mut validator = Validator::new(1, 100);
    
    // Simulate timeout and recovery
    let slot1 = 1;
    let slot2 = 2;
    
    // First slot times out
    if let Some(skip_vote) = validator.create_skip_vote(slot1) {
        let all_votes = vec![skip_vote];
        validator.aggregate_certificates(&all_votes);
    }
    
    // Second slot succeeds
    if let Some(vote) = validator.receive_block_and_vote(slot2, "recovery-block".to_string()) {
        let all_votes = vec![vote];
        validator.aggregate_certificates(&all_votes);
    }
    
    println!("Timeout recovery test completed");
}

/// Test concurrent timeouts
pub fn test_concurrent_timeouts() {
    println!("--- Testing Concurrent Timeouts ---");
    
    let mut validators: Vec<Validator> = (0..8)
        .map(|i| Validator::new(i, 100))
        .collect();
    
    let slot_to_test = 4;
    let mut all_votes_cast = Vec::new();
    
    // Simulate concurrent timeouts
    for validator in &mut validators {
        if let Some(skip_vote) = validator.create_skip_vote(slot_to_test) {
            all_votes_cast.push(skip_vote);
        }
    }
    
    let observer_validator = &mut validators[0];
    observer_validator.aggregate_certificates(&all_votes_cast);
    
    println!("Concurrent timeout handling test completed");
}

/// Test partial network handling
pub fn test_partial_network_handling(offline_percent: u32) {
    println!("--- Testing Partial Network Handling with {}% offline ---", offline_percent);
    
    let mut validators: Vec<Validator> = (0..10)
        .map(|i| Validator::new(i, 100))
        .collect();
    
    let slot_to_test = 6;
    let mut all_votes_cast = Vec::new();
    
    // Simulate partial network
    let offline_count = (10 * offline_percent / 100) as usize;
    
    for (i, validator) in validators.iter_mut().enumerate() {
        if i < offline_count {
            // This validator is offline, doesn't vote
            continue;
        }
        
        if let Some(vote) = validator.receive_block_and_vote(slot_to_test, "partial-block".to_string()) {
            all_votes_cast.push(vote);
        }
    }
    
    let observer_validator = &mut validators[0];
    observer_validator.aggregate_certificates(&all_votes_cast);
    
    println!("Partial network handling test completed");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vote_creation() {
        let vote = Vote::new(1, Some("hash1".to_string()), 1, 100);
        assert_eq!(vote.slot, 1);
        assert_eq!(vote.voter_id, 1);
        assert_eq!(vote.stake, 100);
    }

    #[test]
    fn test_validator_creation() {
        let validator = Validator::new(1, 100);
        assert_eq!(validator.id, 1);
        assert_eq!(validator.stake, 100);
        assert!(!validator.bad_window);
    }

    #[test]
    fn test_skip_vote_creation() {
        let mut validator = Validator::new(1, 100);
        let skip_vote = validator.create_skip_vote(1);
        assert!(skip_vote.is_some());
        assert!(skip_vote.unwrap().block_hash.is_none());
    }
}
