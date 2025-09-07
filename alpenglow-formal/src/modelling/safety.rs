use std::collections::HashMap;

// --- Configuration based on the Alpenglow Whitepaper ---

// The stake threshold required to form a Notarization or Skip Certificate.
const CERTIFICATE_THRESHOLD_PERCENT: u32 = 60;

// We'll model a network with a total of 1000 stake for easy percentage calculation.
const TOTAL_STAKE: u32 = 1000;

// Adversary's power. The protocol is designed to be safe with up to 20%.
const ADVERSARY_STAKE_PERCENT: u32 = 20;

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
    // This dictionary is the key to preventing equivocation for honest nodes.
    // It enforces the "one vote per slot" rule.
    voted_slots: HashMap<u32, String>,
}

impl Validator {
    pub fn new(id: String, stake: u32, is_adversary: bool) -> Self {
        Self {
            id,
            stake,
            is_adversary,
            voted_slots: HashMap::new(),
        }
    }

    pub fn create_vote(&mut self, slot: u32, block_hash: String) -> Option<Vote> {
        if self.is_adversary {
            // Adversaries ignore the rules and will vote for anything.
            println!(" Adversary {} votes for hash '{}' in slot {}.", self.id, block_hash, slot);
            return Some(Vote::new(slot, block_hash, self.id.clone(), self.stake));
        }

        if let Some(previous_hash) = self.voted_slots.get(&slot) {
            // Honest validator's critical safety check.
            println!("✅ Honest Validator {}: IGNORED attempt to vote again for slot {}. Already voted for '{}'.", 
                     self.id, slot, previous_hash);
            return None;
        }
        
        // Record the vote to prevent future votes for this slot.
        self.voted_slots.insert(slot, block_hash.clone());
        println!(" Honest Validator {} votes for hash '{}' in slot {}.", self.id, block_hash, slot);
        Some(Vote::new(slot, block_hash, self.id.clone(), self.stake))
    }
}

pub struct ObserverNode {
    // Stores all votes seen, organized by (slot, hash)
    vote_pool: HashMap<(u32, String), HashMap<String, u32>>,
    formed_certificates: std::collections::HashSet<(u32, String)>,
}

impl ObserverNode {
    pub fn new() -> Self {
        Self {
            vote_pool: HashMap::new(),
            formed_certificates: std::collections::HashSet::new(),
        }
    }

    pub fn receive_vote(&mut self, vote: Option<Vote>) {
        if let Some(vote) = vote {
            let key = vote.key();
            if !self.vote_pool.contains_key(&key) {
                self.vote_pool.insert(key.clone(), HashMap::new());
            }
            // Store vote, ensuring one voter isn't counted twice for the same certificate
            self.vote_pool.get_mut(&key).unwrap().insert(vote.voter_id, vote.stake);
        }
    }

    pub fn aggregate_certificates(&mut self) {
        println!("\n--- Observer Node: Aggregating Certificates ---");
        let required_stake = TOTAL_STAKE * CERTIFICATE_THRESHOLD_PERCENT / 100;
        
        for (key, votes) in &self.vote_pool {
            let current_stake: u32 = votes.values().sum();
            
            println!("  - Checking {:?}: Current Stake = {} / {} required.", key, current_stake, required_stake);
            
            if current_stake >= required_stake {
                self.formed_certificates.insert(key.clone());
                println!("  🔥🔥🔥 CERTIFICATE FORMED for Slot {} with Hash '{}'! Stake: {}", 
                         key.0, key.1, current_stake);
            }
        }
    }

    pub fn get_formed_certificates(&self) -> &std::collections::HashSet<(u32, String)> {
        &self.formed_certificates
    }

    pub fn get_vote_pool(&self) -> &HashMap<(u32, String), HashMap<String, u32>> {
        &self.vote_pool
    }
}

pub fn demonstrate_safety_properties() {
    println!("--- Alpenglow Safety Simulation ---");
    println!("Configuration: {}% Byzantine Stake, {}% Certificate Threshold.\n", 
             ADVERSARY_STAKE_PERCENT, CERTIFICATE_THRESHOLD_PERCENT);

    // 1. SETUP THE NETWORK
    let adversary_stake = TOTAL_STAKE * ADVERSARY_STAKE_PERCENT / 100;
    let honest_stake = TOTAL_STAKE - adversary_stake;
    
    let mut adversary = Validator::new("Adversary-1".to_string(), adversary_stake, true);
    
    // Create a set of honest validators. We'll split them into two groups
    // to simulate a network partition or an attempt by the adversary to split the vote.
    let mut honest_validators_group_a: Vec<Validator> = (0..(honest_stake / 2 / 50))
        .map(|i| Validator::new(format!("Honest-A-{}", i), 50, false))
        .collect();
    let mut honest_validators_group_b: Vec<Validator> = (0..(honest_stake / 2 / 50))
        .map(|i| Validator::new(format!("Honest-B-{}", i), 50, false))
        .collect();

    let slot_to_contest = 10;
    let hash_a = "BLOCK_HASH_ALPHA".to_string();
    let hash_b = "BLOCK_HASH_BRAVO".to_string(); // The conflicting block

    println!("Scenario: Adversary will try to get two conflicting blocks ({}) and ({}) finalized for Slot {}.\n", 
             hash_a, hash_b, slot_to_contest);

    // 2. VOTE GENERATION
    let mut all_votes = Vec::new();

    // Adversary attempts to cause a split:
    // It tells Group A to vote for Hash A, and Group B to vote for Hash B.
    println!("--- Vote Generation Phase ---");
    for validator in &mut honest_validators_group_a {
        all_votes.push(validator.create_vote(slot_to_contest, hash_a.clone()));
    }
    
    for validator in &mut honest_validators_group_b {
        all_votes.push(validator.create_vote(slot_to_contest, hash_b.clone()));
    }

    // **Property: Certificate Uniqueness and Non-Equivocation**
    // The honest validators above have now "locked in" their vote for slot 10.
    // Now, the adversary tries to make them equivocate by showing them the other block.
    println!("\n--- Adversary Attempts to Force Equivocation ---");
    for validator in &mut honest_validators_group_a {
        // These votes will be rejected by the honest validators' internal logic.
        validator.create_vote(slot_to_contest, hash_b.clone()); 
    }
    
    // The adversary itself equivocates, voting for both conflicting blocks.
    println!("\n--- Adversary Equivocates (Votes for Both) ---");
    all_votes.push(adversary.create_vote(slot_to_contest, hash_a.clone()));
    all_votes.push(adversary.create_vote(slot_to_contest, hash_b.clone()));
    
    // 3. AGGREGATION & VERIFICATION
    let mut observer = ObserverNode::new();
    for vote in all_votes {
        observer.receive_vote(vote);
    }
        
    observer.aggregate_certificates();

    // 4. FINAL RESULTS
    println!("\n--- Final Results & Verification ---");
    
    // **Property: Chain consistency under up to 20% Byzantine stake**
    let stake_for_a: u32 = observer.get_vote_pool()
        .get(&(slot_to_contest, hash_a.clone()))
        .map(|votes| votes.values().sum())
        .unwrap_or(0);
    let stake_for_b: u32 = observer.get_vote_pool()
        .get(&(slot_to_contest, hash_b.clone()))
        .map(|votes| votes.values().sum())
        .unwrap_or(0);
    
    println!("Total Stake for Certificate A: {} (400 Honest + 200 Adversary)", stake_for_a);
    println!("Total Stake for Certificate B: {} (400 Honest + 200 Adversary)", stake_for_b);
    println!("Stake Required for Finalization: {}", TOTAL_STAKE * CERTIFICATE_THRESHOLD_PERCENT / 100);
    
    // **Property: No two conflicting blocks can be finalized**
    let finalized_hashes_for_slot: std::collections::HashSet<String> = observer.get_formed_certificates()
        .iter()
        .filter(|cert| cert.0 == slot_to_contest)
        .map(|cert| cert.1.clone())
        .collect();
    
    if finalized_hashes_for_slot.len() > 1 {
        println!("\n🔴🔴🔴 SAFETY FAILURE: Two conflicting blocks were finalized for the same slot!");
    } else if finalized_hashes_for_slot.len() == 1 {
        println!("\n✅ SAFETY HOLDING: Only one block ('{}') was finalized.", 
                 finalized_hashes_for_slot.iter().next().unwrap());
    } else {
        println!("\n✅ SAFETY HOLDING: No certificate reached the threshold. The network stalled but remained safe.");
    }
}

pub fn run_simulation() {
    demonstrate_safety_properties();
}