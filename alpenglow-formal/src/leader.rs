//! Leader rotation and window management for Alpenglow consensus.
//! This module simulates how the system handles leader failures and manages
//! the BadWindow state to maintain safety during periods of liveness issues.

use std::collections::HashMap;
use std::thread;
use std::time::Duration;

// --- Configuration ---
const LEADER_WINDOW_SIZE: u64 = 10; // A smaller window for a quicker simulation

/// Simulates a validator's perspective on leader rotation and window management.
#[derive(Debug)]
pub struct Validator {
    pub id: String,
    #[allow(dead_code)]
    pub stake: u64,
    current_slot: u64,
    bad_window: bool,
    bad_window_triggered_at_slot: Option<u64>,
}

impl Validator {
    pub fn new(id: String, stake: u64) -> Self {
        Self {
            id,
            stake,
            current_slot: 0,
            bad_window: false,
            bad_window_triggered_at_slot: None,
        }
    }

    /// Advances the validator's internal clock.
    pub fn advance_to_slot(&mut self, slot: u64) {
        self.current_slot = slot;
        
        // Check if the reason for the bad window has passed
        if let Some(triggered_slot) = self.bad_window_triggered_at_slot {
            if self.bad_window && self.current_slot >= triggered_slot + LEADER_WINDOW_SIZE {
                println!(
                    "✅ Validator {}: Slot {}. The failure at slot {} is now outside the window. Clearing BadWindow flag.",
                    self.id, self.current_slot, triggered_slot
                );
                self.bad_window = false;
                self.bad_window_triggered_at_slot = None;
            }
        }
    }

    /// Processes a liveness failure (SkipCertificate).
    pub fn process_skip_certificate(&mut self, failed_slot: u64) {
        println!(" Validator {}: Observed SkipCertificate for slot {}.", self.id, failed_slot);
        
        // This is the core window logic
        let window_start = self.current_slot;
        let window_end = self.current_slot + LEADER_WINDOW_SIZE;
        
        if window_start <= failed_slot && failed_slot < window_end {
            if !self.bad_window {
                println!(
                    "🚦 Validator {}: The failure is INSIDE the current window ({}-{}). Setting BadWindow flag to TRUE.",
                    self.id, window_start, window_end - 1
                );
                self.bad_window = true;
                self.bad_window_triggered_at_slot = Some(failed_slot);
            }
        } else {
            println!(" Validator {}: The failure is OUTSIDE the current window. Ignoring.", self.id);
        }
    }

    /// Checks if the validator is allowed to use fast-path mechanisms.
    pub fn can_use_optimistic_path(&self) -> bool {
        !self.bad_window
    }

    #[allow(dead_code)]
    pub fn is_bad_window_active(&self) -> bool {
        self.bad_window
    }
}

/// A simple, deterministic, stake-weighted leader selection function.
/// In reality, this uses a VRF, but this simulates the stake-weighted property.
pub fn get_leader_for_slot(slot: u64, stakes: &HashMap<String, u64>) -> String {
    let total_stake: u64 = stakes.values().sum();
    // This creates a deterministic "random" number for the slot
    let slot_seed = (slot * 1234567891) % total_stake;
    
    let mut cumulative_stake = 0;
    for (validator_id, stake) in stakes {
        cumulative_stake += stake;
        if slot_seed < cumulative_stake {
            return validator_id.clone();
        }
    }
    
    // Fallback to the last validator
    stakes.keys().last().unwrap().clone()
}

/// Runs the leader window simulation.
pub fn run_simulation() {
    println!("--- Alpenglow Leader Window Simulation ---");
    
    let stakes: HashMap<String, u64> = [
        ("Val-A".to_string(), 400),
        ("Val-B".to_string(), 300),
        ("Val-C".to_string(), 200),
        ("Val-D".to_string(), 100),
    ].iter().cloned().collect();
    
    let mut validators: Vec<Validator> = stakes
        .iter()
        .map(|(id, stake)| Validator::new(id.clone(), *stake))
        .collect();
    
    let observer = &mut validators[0]; // We'll watch from this validator's perspective
    
    // Simulate a leader failure at a specific slot
    let slot_with_failure = 15;
    
    println!(
        "Leader window size is {} slots. A leader will fail at slot {}.\n",
        LEADER_WINDOW_SIZE, slot_with_failure
    );

    // Run the simulation slot by slot
    for slot in 0..40 {
        let leader = get_leader_for_slot(slot, &stakes);
        println!("--- Slot {} | Leader: {} ---", slot, leader);
        
        observer.advance_to_slot(slot);
        
        // A liveness failure happens!
        if slot == slot_with_failure {
            println!("🔴 LIVENESS FAILURE: Leader {} for slot {} is offline!", leader, slot);
            observer.process_skip_certificate(slot_with_failure);
        }
        
        // Check the validator's state
        if observer.can_use_optimistic_path() {
            println!("  -> State: Normal. Optimistic paths are ENABLED.");
        } else {
            println!("  -> State: BadWindow Active! Optimistic paths are DISABLED.");
        }
        
        thread::sleep(Duration::from_millis(100));
    }
    
    println!("\n--- Simulation Complete ---");
    println!("The simulation shows how a failure inside the leader window triggers the BadWindow state.");
    println!("This state persists until the window has slid far enough past the failure, allowing the system to automatically recover to its high-performance mode.");
}

/// Test window management functionality
pub fn test_window_management(window_size: u64) {
    println!("--- Testing Window Management with size {} ---", window_size);
    
    let stakes: HashMap<String, u64> = [
        ("Val-A".to_string(), 400),
        ("Val-B".to_string(), 300),
        ("Val-C".to_string(), 200),
        ("Val-D".to_string(), 100),
    ].iter().cloned().collect();
    
    let mut validators: Vec<Validator> = stakes
        .iter()
        .map(|(id, stake)| Validator::new(id.clone(), *stake))
        .collect();
    
    let observer = &mut validators[0];
    
    // Test window management
    for slot in 0..window_size + 5 {
        observer.advance_to_slot(slot);
        let leader = get_leader_for_slot(slot, &stakes);
        
        if slot < window_size {
            println!("Slot {}: Leader {} - Window active", slot, leader);
        } else {
            println!("Slot {}: Leader {} - Window expired", slot, leader);
        }
    }
}

/// Test BadWindow flag management
pub fn test_badwindow_management() {
    println!("--- Testing BadWindow Flag Management ---");
    
    let mut validator = Validator::new("TestVal".to_string(), 100);
    
    // Test BadWindow triggering
    validator.advance_to_slot(10);
    validator.process_skip_certificate(12); // Failure inside window
    assert!(validator.is_bad_window_active());
    
    // Test BadWindow clearing
    validator.advance_to_slot(25); // Move past window
    assert!(!validator.is_bad_window_active());
    
    println!("BadWindow management test completed successfully");
}

/// Test failure handling
pub fn test_failure_handling(failure_rate: u32) {
    println!("--- Testing Failure Handling with {}% failure rate ---", failure_rate);
    
    let stakes: HashMap<String, u64> = [
        ("Val-A".to_string(), 400),
        ("Val-B".to_string(), 300),
        ("Val-C".to_string(), 200),
        ("Val-D".to_string(), 100),
    ].iter().cloned().collect();
    
    let mut validators: Vec<Validator> = stakes
        .iter()
        .map(|(id, stake)| Validator::new(id.clone(), *stake))
        .collect();
    
    let observer = &mut validators[0];
    
    // Simulate failures
    for slot in 0..20 {
        observer.advance_to_slot(slot);
        let leader = get_leader_for_slot(slot, &stakes);
        
        // Simulate failure based on rate
        if (slot as u32) % 100 < failure_rate {
            println!("Slot {}: Leader {} FAILED", slot, leader);
            observer.process_skip_certificate(slot);
        } else {
            println!("Slot {}: Leader {} SUCCESS", slot, leader);
        }
    }
}

/// Test stake-weighted selection
pub fn test_stake_weighted_selection() {
    println!("--- Testing Stake-Weighted Selection ---");
    
    let stakes: HashMap<String, u64> = [
        ("Val-A".to_string(), 400),
        ("Val-B".to_string(), 300),
        ("Val-C".to_string(), 200),
        ("Val-D".to_string(), 100),
    ].iter().cloned().collect();
    
    // Test multiple slots to verify distribution
    let mut leader_counts: HashMap<String, u32> = HashMap::new();
    
    for slot in 0..100 {
        let leader = get_leader_for_slot(slot, &stakes);
        *leader_counts.entry(leader).or_insert(0) += 1;
    }
    
    println!("Leader selection distribution over 100 slots:");
    for (leader, count) in &leader_counts {
        println!("  {}: {} times", leader, count);
    }
}

/// Test window sliding
pub fn test_window_sliding() {
    println!("--- Testing Window Sliding ---");
    
    let mut validator = Validator::new("TestVal".to_string(), 100);
    
    // Test window sliding behavior
    for slot in 0..30 {
        validator.advance_to_slot(slot);
        
        // Trigger failure at slot 10
        if slot == 10 {
            validator.process_skip_certificate(10);
        }
        
        // Check if BadWindow is active
        if validator.is_bad_window_active() {
            println!("Slot {}: BadWindow ACTIVE", slot);
        } else {
            println!("Slot {}: BadWindow CLEAR", slot);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = Validator::new("TestVal".to_string(), 100);
        assert_eq!(validator.id, "TestVal");
        assert_eq!(validator.stake, 100);
        assert!(!validator.bad_window);
    }

    #[test]
    fn test_leader_selection() {
        let stakes: HashMap<String, u64> = [
            ("Val-A".to_string(), 400),
            ("Val-B".to_string(), 300),
        ].iter().cloned().collect();
        
        let leader = get_leader_for_slot(0, &stakes);
        assert!(stakes.contains_key(&leader));
    }

    #[test]
    fn test_bad_window_logic() {
        let mut validator = Validator::new("TestVal".to_string(), 100);
        validator.advance_to_slot(10);
        validator.process_skip_certificate(15); // Failure outside window
        assert!(!validator.bad_window);
        
        validator.process_skip_certificate(12); // Failure inside window
        assert!(validator.bad_window);
    }
}
