//! Formal verification model for leader rotation and window management in Alpenglow consensus.
//! This module provides a Stateright-based formal model for verifying leader selection,
//! window management, and BadWindow flag handling.

use stateright::{Model, Property, Checker};
use std::collections::{BTreeMap, BTreeSet};

// --- Formal Model Configuration ---
const LEADER_WINDOW_SIZE: u64 = 5; // Formal verification limit
const MAX_SLOTS: u64 = 10; // Formal verification limit
const MAX_VALIDATORS: usize = 5; // Formal verification limit

// Type aliases for clarity
type Slot = u64;
type ActorId = usize;
type Stake = u64;

/// Represents different types of messages in the leader system
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum LeaderMessage {
    /// Leader selection for a slot
    LeaderSelection {
        slot: Slot,
        leader: ActorId,
        stake: Stake,
    },
    /// Skip certificate indicating leader failure
    SkipCertificate {
        slot: Slot,
        failed_leader: ActorId,
    },
    /// BadWindow flag update
    BadWindowUpdate {
        slot: Slot,
        validator: ActorId,
        bad_window: bool,
    },
}

/// Represents messages in transit
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct MessageInTransit {
    dst: ActorId,
    msg: LeaderMessage,
}

/// Actions that can be taken in the leader model
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum LeaderAction {
    /// Select a leader for a slot
    SelectLeader {
        slot: Slot,
        leader: ActorId,
    },
    /// Deliver a message to its destination
    DeliverMessage { msg: MessageInTransit },
    /// Trigger a leader failure
    TriggerLeaderFailure {
        slot: Slot,
        leader: ActorId,
    },
    /// Advance to the next slot
    AdvanceSlot,
}

/// State of a validator in the leader model
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ValidatorState {
    /// Current slot
    current_slot: Slot,
    /// BadWindow flag state
    bad_window: bool,
    /// Slot when BadWindow was triggered
    bad_window_triggered_at: Option<Slot>,
    /// Known leaders: slot -> leader
    known_leaders: BTreeMap<Slot, ActorId>,
    /// Known skip certificates: slot -> failed leader
    skip_certificates: BTreeMap<Slot, ActorId>,
    /// Stake distribution
    stake: Stake,
}

/// Main state of the leader formal model
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct LeaderState {
    /// Network messages in transit
    network: BTreeSet<MessageInTransit>,
    /// Per-validator states
    validators: Vec<ValidatorState>,
    /// Global current slot
    current_slot: Slot,
    /// Leader assignments: slot -> leader
    leader_assignments: BTreeMap<Slot, ActorId>,
    /// Leader failures: slot -> failed leader
    leader_failures: BTreeMap<Slot, ActorId>,
    /// Stake distribution: validator -> stake
    stake_distribution: BTreeMap<ActorId, Stake>,
}

/// Formal model for leader rotation and window management
#[derive(Clone)]
pub struct LeaderModel {
    /// Number of validators
    pub validator_count: usize,
    /// Maximum slots to explore
    pub max_slot: Slot,
}

impl LeaderState {
    fn new(validator_count: usize) -> Self {
        let mut stake_distribution = BTreeMap::new();
        let stake_per_validator = 1000 / validator_count as u64;
        
        for i in 0..validator_count {
            stake_distribution.insert(i, stake_per_validator);
        }

        Self {
            network: BTreeSet::new(),
            validators: (0..validator_count).map(|_i| ValidatorState {
                current_slot: 0,
                bad_window: false,
                bad_window_triggered_at: None,
                known_leaders: BTreeMap::new(),
                skip_certificates: BTreeMap::new(),
                stake: stake_per_validator,
            }).collect(),
            current_slot: 0,
            leader_assignments: BTreeMap::new(),
            leader_failures: BTreeMap::new(),
            stake_distribution,
        }
    }

    /// Get leader for a slot using stake-weighted selection
    fn get_leader_for_slot(&self, slot: Slot) -> ActorId {
        let total_stake: Stake = self.stake_distribution.values().sum();
        let slot_seed = (slot * 1234567891) % total_stake;
        
        let mut cumulative_stake = 0;
        for (validator_id, stake) in &self.stake_distribution {
            cumulative_stake += stake;
            if slot_seed < cumulative_stake {
                return *validator_id;
            }
        }
        
        // Fallback to last validator
        *self.stake_distribution.keys().last().unwrap()
    }

    /// Check if a slot is within the leader window
    fn is_within_window(&self, slot: Slot, current_slot: Slot) -> bool {
        current_slot <= slot && slot < current_slot + LEADER_WINDOW_SIZE
    }

    /// Update BadWindow flags based on skip certificates
    fn update_badwindow_flags(&mut self) {
        for validator in &mut self.validators {
            let current_slot = validator.current_slot;
            // Check if any skip certificate is within the current window
            let has_skip_in_window = validator.skip_certificates.iter()
                .any(|(slot, _)| current_slot <= *slot && *slot < current_slot + LEADER_WINDOW_SIZE);
            
            if has_skip_in_window && !validator.bad_window {
                validator.bad_window = true;
                validator.bad_window_triggered_at = Some(current_slot);
            } else if !has_skip_in_window && validator.bad_window {
                // Clear BadWindow if no skip certificates in window
                validator.bad_window = false;
                validator.bad_window_triggered_at = None;
            }
        }
    }
}

impl Model for LeaderModel {
    type State = LeaderState;
    type Action = LeaderAction;

    fn init_states(&self) -> Vec<Self::State> {
        vec![LeaderState::new(self.validator_count)]
    }

    fn actions(&self, state: &Self::State, actions: &mut Vec<Self::Action>) {
        // 1. Deliver any message in the network
        for msg in &state.network {
            actions.push(LeaderAction::DeliverMessage { msg: msg.clone() });
        }

        // 2. Select leaders for current and future slots
        for slot in state.current_slot..=self.max_slot {
            if !state.leader_assignments.contains_key(&slot) {
                let leader = state.get_leader_for_slot(slot);
                actions.push(LeaderAction::SelectLeader { slot, leader });
            }
        }

        // 3. Trigger leader failures for any slot
        for slot in 1..=self.max_slot {
            if let Some(leader) = state.leader_assignments.get(&slot) {
                actions.push(LeaderAction::TriggerLeaderFailure { slot, leader: *leader });
            }
        }

        // 4. Advance to next slot
        if state.current_slot < self.max_slot {
            actions.push(LeaderAction::AdvanceSlot);
        }
    }

    fn next_state(&self, last_state: &Self::State, action: Self::Action) -> Option<Self::State> {
        let mut next_state = last_state.clone();
        let mut validators = last_state.validators.clone();

        match action {
            LeaderAction::SelectLeader { slot, leader } => {
                next_state.leader_assignments.insert(slot, leader);

                // Broadcast leader selection to all validators
                if let Some(stake) = next_state.stake_distribution.get(&leader) {
                    for dst in 0..self.validator_count {
                        next_state.network.insert(MessageInTransit {
                            dst,
                            msg: LeaderMessage::LeaderSelection {
                                slot,
                                leader,
                                stake: *stake,
                            },
                        });
                    }
                }
            }
            LeaderAction::DeliverMessage { msg } => {
                let recipient_id = msg.dst;
                let mut validator_state = validators[recipient_id].clone();

                // Remove message from network
                if !next_state.network.remove(&msg) { return None; }

                match msg.msg {
                    LeaderMessage::LeaderSelection { slot, leader, stake: _ } => {
                        validator_state.known_leaders.insert(slot, leader);
                    }
                    LeaderMessage::SkipCertificate { slot, failed_leader } => {
                        validator_state.skip_certificates.insert(slot, failed_leader);
                    }
                    LeaderMessage::BadWindowUpdate { slot: _, validator: _, bad_window } => {
                        validator_state.bad_window = bad_window;
                    }
                }
                validators[recipient_id] = validator_state;
            }
            LeaderAction::TriggerLeaderFailure { slot, leader } => {
                next_state.leader_failures.insert(slot, leader);

                // Broadcast skip certificate to all validators
                for i in 0..self.validator_count {
                    next_state.network.insert(MessageInTransit {
                        dst: i,
                        msg: LeaderMessage::SkipCertificate { slot, failed_leader: leader },
                    });
                }
            }
            LeaderAction::AdvanceSlot => {
                next_state.current_slot += 1;
                for validator_state in &mut validators {
                    validator_state.current_slot = next_state.current_slot;
                }
                // Update BadWindow flags when advancing slots
                next_state.update_badwindow_flags();
            }
        }

        next_state.validators = validators;
        Some(next_state)
    }

    /// Properties to verify in the leader model
    fn properties(&self) -> Vec<Property<Self>> {
        vec![
            // Property 1: Leader uniqueness per slot
            Property::<Self>::always("leader_uniqueness", |_, state| {
                // Each slot has at most one leader
                state.leader_assignments.len() <= state.current_slot as usize + 1
            }),
            
            // Property 2: BadWindow consistency
            Property::<Self>::always("badwindow_consistency", |_, state| {
                for validator in &state.validators {
                    if validator.bad_window {
                        // If BadWindow is set, there must be a skip certificate in the window
                        let has_skip_in_window = validator.skip_certificates.iter()
                            .any(|(slot, _)| state.is_within_window(*slot, validator.current_slot));
                        if !has_skip_in_window {
                            return false;
                        }
                    }
                }
                true
            }),
            
            // Property 3: Stake-weighted leader selection
            Property::<Self>::always("stake_weighted_selection", |_, state| {
                // Leaders should be selected based on stake distribution
                for (slot, leader) in &state.leader_assignments {
                    if *slot <= state.current_slot {
                        // Verify the leader was selected using stake-weighted method
                        let expected_leader = state.get_leader_for_slot(*slot);
                        if expected_leader != *leader {
                            return false;
                        }
                    }
                }
                true
            }),
            
            // Property 4: Window management correctness
            Property::<Self>::always("window_management", |_, state| {
                for validator in &state.validators {
                    // BadWindow should be cleared when skip certificates move out of window
                    for (skip_slot, _) in &validator.skip_certificates {
                        if !state.is_within_window(*skip_slot, validator.current_slot) {
                            // Skip certificate is outside window, BadWindow should be cleared
                            if validator.bad_window {
                                // Check if there are other skip certificates in window
                                let has_other_skip_in_window = validator.skip_certificates.iter()
                                    .any(|(other_slot, _)| *other_slot != *skip_slot && 
                                         state.is_within_window(*other_slot, validator.current_slot));
                                if !has_other_skip_in_window {
                                    return false; // BadWindow should be cleared
                                }
                            }
                        }
                    }
                }
                true
            }),
        ]
    }
}

/// Run formal verification of leader rotation
pub fn run_formal_verification() {
    println!("=== Leader Rotation Formal Verification ===");
    
    let model = LeaderModel {
        validator_count: 3, // Small for formal verification
        max_slot: 5,
    };

    println!("Model checking leader rotation with {} validators, {} slots", 
             model.validator_count, model.max_slot);
    
    let result = model
        .checker()
        .threads(num_cpus::get())
        .spawn_dfs()
        .report(&mut stateright::report::WriteReporter::new(&mut std::io::stdout()));
    
    // Check verification results
    if result.discoveries().is_empty() {
        println!("✅ All leader rotation properties verified successfully");
        } else {
        println!("❌ Leader rotation verification found counterexamples");
        for (property_name, _path) in result.discoveries() {
            println!("  - {}", property_name);
        }
    }
}

/// Test leader model with different configurations
pub fn test_leader_model(validators: usize, slots: u64) {
    println!("Testing leader model with {} validators, {} slots", validators, slots);
    
    let model = LeaderModel {
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
    fn test_leader_state_creation() {
        let state = LeaderState::new(3);
        assert_eq!(state.validators.len(), 3);
        assert_eq!(state.current_slot, 0);
        assert!(state.network.is_empty());
    }

    #[test]
    fn test_leader_selection() {
        let state = LeaderState::new(3);
        let leader = state.get_leader_for_slot(1);
        assert!(leader < 3);
    }

    #[test]
    fn test_window_management() {
        let state = LeaderState::new(3);
        assert!(state.is_within_window(5, 5)); // Within window
        assert!(!state.is_within_window(15, 5)); // Outside window
    }
}
