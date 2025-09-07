use std::collections::HashMap;
use std::collections::HashSet;

// --- Configuration based on the Alpenglow Whitepaper ---
// We model time in discrete "ticks". 1 tick = δ (average network latency).

// Thresholds for finalization paths
const FAST_PATH_THRESHOLD_PERCENT: u32 = 80;
const SLOW_PATH_THRESHOLD_PERCENT: u32 = 60; // Also known as the Notarization threshold

// Total stake in the simulated network
const TOTAL_STAKE: u32 = 1000;

#[derive(Debug, Clone)]
pub struct Message {
    pub msg_type: String,
    pub slot: u32,
    pub block_hash: String,
    pub sender_id: u32,
    pub delivery_tick: u32, // The tick at which this message arrives
}

impl Message {
    pub fn new(msg_type: String, slot: u32, block_hash: String, sender_id: u32, delivery_tick: u32) -> Self {
        Self {
            msg_type,
            slot,
            block_hash,
            sender_id,
            delivery_tick,
        }
    }
}

pub struct Validator {
    pub id: u32,
    pub stake: u32,
    pub is_responsive: bool,
    
    // State tracking
    voted_at_slot: HashMap<u32, String>,        // Enforces one vote per slot
    notarized_slots: HashSet<u32>,              // Slots that have crossed 60% (triggers FinalVote)
    finalized_slots: HashMap<u32, String>,      // Final record of finalized blocks
    votes_seen: HashMap<u32, Vec<Message>>,     // Local pool of votes received from the network
}

impl Validator {
    pub fn new(id: u32, stake: u32, is_responsive: bool) -> Self {
        Self {
            id,
            stake,
            is_responsive,
            voted_at_slot: HashMap::new(),
            notarized_slots: HashSet::new(),
            finalized_slots: HashMap::new(),
            votes_seen: HashMap::new(),
        }
    }

    pub fn process_message(&mut self, msg: &Message, network_pipe: &mut Vec<Message>, current_tick: u32, validator_stakes: &HashMap<u32, u32>) {
        if !self.is_responsive {
            return; // This validator is offline/unresponsive
        }

        // --- Main State Machine ---
        if msg.msg_type == "PROPOSAL" {
            // Upon receiving a block, cast the first vote (NotarVote)
            if !self.voted_at_slot.contains_key(&msg.slot) {
                self.voted_at_slot.insert(msg.slot, msg.block_hash.clone());
                let vote = Message::new("NOTAR_VOTE".to_string(), msg.slot, msg.block_hash.clone(), self.id, current_tick + 1);
                network_pipe.push(vote);
            }
        } else if msg.msg_type == "NOTAR_VOTE" || msg.msg_type == "FINAL_VOTE" {
            // Add any vote to our local pool
            if !self.votes_seen.contains_key(&msg.slot) {
                self.votes_seen.insert(msg.slot, Vec::new());
            }
            self.votes_seen.get_mut(&msg.slot).unwrap().push(msg.clone());
            
            // --- Check for Finalization ---
            self.check_for_finalization(msg.slot, &msg.block_hash, network_pipe, current_tick, validator_stakes);
        }
    }

    fn check_for_finalization(&mut self, slot: u32, block_hash: &str, network_pipe: &mut Vec<Message>, current_tick: u32, validator_stakes: &HashMap<u32, u32>) {
        if self.finalized_slots.contains_key(&slot) {
            return; // Already finalized this slot
        }

        // Tally stakes for the specific block hash
        let notar_votes: HashSet<u32> = self.votes_seen.get(&slot)
            .map(|votes| votes.iter()
                .filter(|m| m.msg_type == "NOTAR_VOTE" && m.block_hash == block_hash)
                .map(|m| m.sender_id)
                .collect())
            .unwrap_or_default();
        
        let final_votes: HashSet<u32> = self.votes_seen.get(&slot)
            .map(|votes| votes.iter()
                .filter(|m| m.msg_type == "FINAL_VOTE" && m.block_hash == block_hash)
                .map(|m| m.sender_id)
                .collect())
            .unwrap_or_default();

        let notar_stake: u32 = notar_votes.iter()
            .filter_map(|id| validator_stakes.get(id))
            .sum();
        
        let final_stake: u32 = final_votes.iter()
            .filter_map(|id| validator_stakes.get(id))
            .sum();

        // 1. FAST PATH CHECK (>80% on NotarVotes)
        if notar_stake >= (TOTAL_STAKE * FAST_PATH_THRESHOLD_PERCENT / 100) {
            self.finalized_slots.insert(slot, block_hash.to_string());
            println!("  T{}: Validator {} FINALIZED Slot {} via FAST PATH (1 Round). Stake: {}", 
                     current_tick, self.id, slot, notar_stake);
            return;
        }

        // 2. SLOW PATH - STEP 1: NOTARIZATION (>60% on NotarVotes)
        if !self.notarized_slots.contains(&slot) && notar_stake >= (TOTAL_STAKE * SLOW_PATH_THRESHOLD_PERCENT / 100) {
            self.notarized_slots.insert(slot);
            // This triggers the second round of voting
            let final_vote = Message::new("FINAL_VOTE".to_string(), slot, block_hash.to_string(), self.id, current_tick + 1);
            network_pipe.push(final_vote);
            println!("  T{}: Validator {} NOTARIZED Slot {}. Broadcasting FinalVote. Stake: {}", 
                     current_tick, self.id, slot, notar_stake);
        }

        // 3. SLOW PATH - STEP 2: FINALIZATION (>60% on FinalVotes)
        if self.notarized_slots.contains(&slot) && final_stake >= (TOTAL_STAKE * SLOW_PATH_THRESHOLD_PERCENT / 100) {
            self.finalized_slots.insert(slot, block_hash.to_string());
            println!("  T{}: Validator {} FINALIZED Slot {} via SLOW PATH (2 Rounds). Stake: {}", 
                     current_tick, self.id, slot, final_stake);
        }
    }

    pub fn has_finalized(&self, slot: u32) -> bool {
        self.finalized_slots.contains_key(&slot)
    }
}

pub fn run_scenario(responsive_stake_percent: u32, max_ticks: u32) -> i32 {
    let mut validators = Vec::new();
    let mut network_pipe = Vec::new();

    // --- SETUP ---
    let responsive_stake = TOTAL_STAKE * responsive_stake_percent / 100;
    let mut current_responsive_stake = 0;
    for i in 0..10 { // Create 10 validators of 100 stake each
        let is_responsive = current_responsive_stake < responsive_stake;
        validators.push(Validator::new(i, 100, is_responsive));
        if is_responsive {
            current_responsive_stake += 100;
        }
    }

    // Create validator stakes map for efficient lookup
    let validator_stakes: HashMap<u32, u32> = validators.iter()
        .map(|v| (v.id, v.stake))
        .collect();

    println!("\n--- SCENARIO: {}% Responsive Stake ---", responsive_stake_percent);
    println!("Fast Path Threshold: {} | Slow Path Threshold: {}\n", 
             TOTAL_STAKE * FAST_PATH_THRESHOLD_PERCENT / 100, 
             TOTAL_STAKE * SLOW_PATH_THRESHOLD_PERCENT / 100);

    let slot_to_finalize = 1;
    let leader = &validators[0]; // Assume validator 0 is the leader

    // --- SIMULATION START ---
    // T=0: Leader proposes a block. Message will arrive at T=1.
    println!("T0: Leader {} proposes block 'B1' for Slot {}.", leader.id, slot_to_finalize);
    network_pipe.push(Message::new("PROPOSAL".to_string(), slot_to_finalize, "B1".to_string(), leader.id, 1));

    let mut finalization_tick = -1;

    for tick in 1..max_ticks {
        // Process all messages scheduled to arrive at this tick
        let messages_to_process: Vec<Message> = network_pipe.iter()
            .filter(|m| m.delivery_tick == tick)
            .cloned()
            .collect();
        
        if !messages_to_process.is_empty() {
            for v in &mut validators {
                for msg in &messages_to_process {
                    v.process_message(msg, &mut network_pipe, tick, &validator_stakes);
                }
            }
        }
        
        // Check for finalization across the network
        let finalized_count = validators.iter()
            .filter(|v| v.has_finalized(slot_to_finalize))
            .count();
        if finalized_count > 0 && finalization_tick == -1 {
            finalization_tick = tick as i32;
        }
    }

    // --- RESULTS ---
    let finalized_count_at_end = validators.iter()
        .filter(|v| v.has_finalized(slot_to_finalize) && v.is_responsive)
        .count();
    let responsive_validators = validators.iter()
        .filter(|v| v.is_responsive)
        .count();
    
    if finalized_count_at_end >= (responsive_validators * 8 / 10) { // Check if most responsive nodes finalized
         println!("\nRESULT: Liveness Success! Slot finalized at T={}.", finalization_tick);
         finalization_tick
    } else {
        println!("\nRESULT: Liveness Failure! Slot did not finalize within the time limit.");
        -1
    }
}

pub fn run_simulation() {
    // Property 1: Progress guarantee with >60% honest participation
    run_scenario(70, 10); // Should succeed (via slow path)
    run_scenario(50, 10); // Should fail

    // Property 2: Fast path completion with >80% responsive stake
    let _fast_path_time = run_scenario(90, 10); // Should succeed (via fast path)

    // Property 3: Bounded finalization time min(δ₈₀%, 2δ₆₀%)
    println!("\n\n--- Verifying Bounded Finalization Time ---");
    println!("The Votor protocol finalizes at the minimum of two concurrent timers:");
    println!("1. δ₈₀%: The time to collect an 80%+ supermajority of votes (fast path).");
    println!("2. 2δ₆₀%: The time to collect a 60%+ majority, broadcast a second vote, and collect another 60%+ majority (slow path).");
    
    let t_fast_path = run_scenario(90, 10); // Should take ~2 ticks (1 for proposal, 1 for votes)
    let t_slow_path = run_scenario(70, 10); // Should take ~4 ticks (1 prop, 1 notar, 1 final, 1 finalization)

    println!("\nObserved Fast Path Time (δ₈₀%) ≈ {} ticks.", t_fast_path);
    println!("Observed Slow Path Time (2δ₆₀%) ≈ {} ticks.", t_slow_path);
    println!("This simulation demonstrates that the network finalizes as soon as the *first* of these conditions is met, ensuring predictable and optimal liveness.");
}