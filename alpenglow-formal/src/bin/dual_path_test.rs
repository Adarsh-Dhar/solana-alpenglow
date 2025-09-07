use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let mut path = "fast";
    let mut threshold = 80;
    let mut stake_percent = 90;
    let mut seed = 12345;
    
    for i in 0..args.len() {
        if args[i] == "--path" && i + 1 < args.len() {
            path = &args[i + 1];
        } else if args[i] == "--threshold" && i + 1 < args.len() {
            threshold = args[i + 1].parse().unwrap_or(80);
        } else if args[i] == "--stake-percent" && i + 1 < args.len() {
            stake_percent = args[i + 1].parse().unwrap_or(90);
        } else if args[i] == "--seed" && i + 1 < args.len() {
            seed = args[i + 1].parse().unwrap_or(12345);
        }
    }
    
    println!("Running dual path test: {} path, {}% threshold, {}% stake, seed {}", path, threshold, stake_percent, seed);
    
    // Simulate dual path finality
    if path == "fast" {
        if stake_percent >= threshold {
            println!("Fast path finalization successful");
            println!("Finalization time: 1");
            println!("Rounds completed: 1");
        } else {
            println!("Fast path finalization failed - insufficient stake");
        }
    } else if path == "slow" {
        if stake_percent >= 60 {
            println!("Slow path finalization successful");
            println!("Finalization time: 2");
            println!("Rounds completed: 2");
        } else {
            println!("Slow path finalization failed - insufficient stake");
        }
    } else {
        println!("Unknown path type: {}", path);
    }
}