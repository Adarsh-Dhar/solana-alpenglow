use std::collections::HashMap;
use std::collections::HashSet;

// --- Configuration based on Alpenglow Guarantees ---
const CERTIFICATE_THRESHOLD_PERCENT: u32 = 60;
const TOTAL_STAKE: u32 = 1000;

#[derive(Debug, Clone)]
pub struct Vote {
    pub slot: u32,
    pub block_hash: String,
    pub voter_id: String,
    pub stake: u32,
}

impl Vote {
    pub fn new(slot: u32, block_hash: String, voter_id: String, stake: u32) -> Self {
        Self {
            slot,
            block_hash,
            voter_id,
            stake,
        }
    }

    pub fn key(&self) -> (u32, String) {
        (self.slot, self.block_hash.clone())
    }
}

pub struct Validator {
    pub id: String,
    pub stake: u32,
    pub is_adversary: bool,
    pub is_responsive: bool,
    voted_slots: HashMap<u32, String>,
}

impl Validator {
    pub fn new(id: String, stake: u32, is_adversary: bool, is_responsive: bool) -> Self {
        Self {
            id,
            stake,
            is_adversary,
            is_responsive,
            voted_slots: HashMap::new(),
        }
    }

    pub fn create_vote(&mut self, slot: u32, block_hash: String) -> Option<Vote> {
        if !self.is_responsive {
            return None; // Offline nodes don't vote
        }

        if self.is_adversary {
            // Adversary will happily equivocate (vote for multiple conflicting blocks)
            return Some(Vote::new(slot, block_hash, self.id.clone(), self.stake));
        }

        if self.voted_slots.contains_key(&slot) {
            // Honest validator's safety rule: NEVER vote twice for the same slot.
            return None;
        }
        
        self.voted_slots.insert(slot, block_hash.clone());
        Some(Vote::new(slot, block_hash, self.id.clone(), self.stake))
    }
}

pub fn aggregate_certificates(votes: &[Option<Vote>]) -> HashSet<(u32, String)> {
    let mut vote_pool: HashMap<(u32, String), HashMap<String, u32>> = HashMap::new();
    
    for vote in votes.iter().flatten() {
        let key = vote.key();
        if !vote_pool.contains_key(&key) {
            vote_pool.insert(key.clone(), HashMap::new());
        }
        vote_pool.get_mut(&key).unwrap().insert(vote.voter_id.clone(), vote.stake);
    }
        
    let mut formed_certificates = HashSet::new();
    let required_stake = TOTAL_STAKE * CERTIFICATE_THRESHOLD_PERCENT / 100;
    
    for (key, voters) in vote_pool {
        let current_stake: u32 = voters.values().sum();
        if current_stake >= required_stake {
            formed_certificates.insert(key);
        }
    }
    formed_certificates
}

pub fn simulate_safety_under_attack(adversary_stake_percent: u32) {
    println!("\n--- SCENARIO 1: Safety with {}% Byzantine Stake ---", adversary_stake_percent);
    
    let mut adversary = Validator::new("Adversary".to_string(), TOTAL_STAKE * adversary_stake_percent / 100, true, true);
    
    let honest_stake_total = TOTAL_STAKE - adversary.stake;
    // Split honest nodes into two groups to simulate the adversary's attempt
    let mut honest_group_a: Vec<Validator> = (0..(honest_stake_total / 2 / 50))
        .map(|i| Validator::new(format!("Honest-A-{}", i), 50, false, true))
        .collect();
    let mut honest_group_b: Vec<Validator> = (0..(honest_stake_total / 2 / 50))
        .map(|i| Validator::new(format!("Honest-B-{}", i), 50, false, true))
        .collect();
    
    let slot = 10;
    let hash_a = "BLOCK_A".to_string();
    let hash_b = "BLOCK_B".to_string(); // Conflicting block
    
    println!("Adversary will try to get two certificates for Slot {} ('{}' and '{}').", slot, hash_a, hash_b);
    
    let mut votes = Vec::new();
    // Adversary tells Group A to vote for A, and Group B to vote for B
    for v in &mut honest_group_a {
        votes.push(v.create_vote(slot, hash_a.clone()));
    }
    for v in &mut honest_group_b {
        votes.push(v.create_vote(slot, hash_b.clone()));
    }
    
    // Adversary equivocates and votes for both
    votes.push(adversary.create_vote(slot, hash_a.clone()));
    votes.push(adversary.create_vote(slot, hash_b.clone()));
    
    let certificates = aggregate_certificates(&votes);
    
    println!("\nRESULT:");
    let finalized_hashes: HashSet<String> = certificates.iter()
        .filter(|cert| cert.0 == slot)
        .map(|cert| cert.1.clone())
        .collect();
    
    if finalized_hashes.len() > 1 {
        println!("🔴 FAILURE: Safety violated! Two conflicting certificates were formed.");
    } else {
        println!("✅ SUCCESS: Safety maintained. It's impossible to form two conflicting certificates.");
        println!("   The adversary can't force honest nodes to vote twice, and the remaining stake is insufficient to reach the 60% threshold for two separate blocks.");
    }
}

pub fn simulate_liveness_with_offline_nodes(offline_stake_percent: u32) {
    println!("\n--- SCENARIO 2: Liveness with {}% Non-Responsive Stake ---", offline_stake_percent);
    
    let responsive_stake = TOTAL_STAKE * (100 - offline_stake_percent) / 100;
    let mut validators = Vec::new();
    let mut current_stake = 0;
    while current_stake < TOTAL_STAKE {
        let is_responsive = current_stake < responsive_stake;
        validators.push(Validator::new(format!("V-{}", current_stake / 100), 100, false, is_responsive));
        current_stake += 100;
    }

    println!("Total responsive stake: {}/{}", responsive_stake, TOTAL_STAKE);
    println!("Required for progress: {}", TOTAL_STAKE * CERTIFICATE_THRESHOLD_PERCENT / 100);
    
    let slot = 20;
    let block_hash = "BLOCK_C".to_string();
    
    // Leader proposes a block, and all responsive validators vote for it.
    let votes: Vec<Option<Vote>> = validators.iter_mut()
        .map(|v| v.create_vote(slot, block_hash.clone()))
        .collect();
    
    let certificates = aggregate_certificates(&votes);
    
    println!("\nRESULT:");
    if certificates.iter().any(|cert| cert.0 == slot) {
        println!("✅ SUCCESS: Liveness maintained. The responsive stake was sufficient to form a certificate and finalize the block.");
    } else {
        println!("🔴 FAILURE: Liveness stalled. Not enough responsive stake to make progress.");
    }
}

pub fn simulate_network_partition_recovery() {
    println!("\n--- SCENARIO 3: Network Partition Recovery ---");
    
    // Split the network 50/50. Neither side has a supermajority.
    let mut partition_a: Vec<Validator> = (0..5)
        .map(|i| Validator::new(format!("Part-A-{}", i), 100, false, true))
        .collect();
    let mut partition_b: Vec<Validator> = (0..5)
        .map(|i| Validator::new(format!("Part-B-{}", i), 100, false, true))
        .collect();
    
    let slot = 30;
    let hash_a = "BLOCK_PART_A".to_string();
    
    // --- PHASE 1: Network is Partitioned ---
    println!("\nPHASE 1: Network is partitioned. Validators can only see votes from their own partition.");
    
    // A leader in Partition A proposes a block. Only Partition A votes.
    let votes_a: Vec<Option<Vote>> = partition_a.iter_mut()
        .map(|v| v.create_vote(slot, hash_a.clone()))
        .collect();
    
    let certs_a = aggregate_certificates(&votes_a);
    println!("Partition A Certificates: {:?}", certs_a);
    
    // Partition B sees nothing and may vote to skip (not simulated for brevity)
    // The key is they cannot contribute to Partition A's block.
    
    println!("\nRESULT (During Partition):");
    if certs_a.is_empty() {
        println!("✅ SUCCESS (Safety): No certificate was formed in Partition A as it lacks a 60% supermajority.");
        println!("   The network is stalled but safe from a chain split.");
    } else {
        println!("🔴 FAILURE: A minority partition was able to finalize a block!");
    }

    // --- PHASE 2: Partition Heals ---
    println!("\nPHASE 2: Partition heals. All nodes can now see all votes.");
    
    // We assume Partition B also saw the proposal for BLOCK_PART_A and would have voted for it.
    // In reality, they would sync state, but this simulates the end result.
    let votes_b: Vec<Option<Vote>> = partition_b.iter_mut()
        .map(|v| v.create_vote(slot, hash_a.clone()))
        .collect();
    let mut all_votes = votes_a;
    all_votes.extend(votes_b);
    
    let all_certificates = aggregate_certificates(&all_votes);
    
    println!("\nRESULT (After Healing):");
    if all_certificates.iter().any(|cert| cert.0 == slot) {
        println!("✅ SUCCESS (Recovery): A certificate was formed after healing!");
        println!("   The network successfully recovered and can now make progress on a single, unified chain.");
    } else {
        println!("🔴 FAILURE: Network failed to recover after the partition healed.");
    }
}

pub fn run_simulation() {
    simulate_safety_under_attack(20);
    simulate_liveness_with_offline_nodes(20);
    simulate_network_partition_recovery();
}