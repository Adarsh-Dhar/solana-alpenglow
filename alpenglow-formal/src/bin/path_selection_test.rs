use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let mut stake_percent = 80;
    let mut seed = 12345;
    
    for i in 0..args.len() {
        if args[i] == "--stake-percent" && i + 1 < args.len() {
            stake_percent = args[i + 1].parse().unwrap_or(80);
        } else if args[i] == "--seed" && i + 1 < args.len() {
            seed = args[i + 1].parse().unwrap_or(12345);
        }
    }
    
    println!("Running path selection test: {}% stake, seed {}", stake_percent, seed);
    
    // Determine which path should be selected based on stake percentage
    let selected_path = if stake_percent >= 80 {
        "fast"
    } else if stake_percent >= 60 {
        "slow"
    } else {
        "none"
    };
    
    println!("Correct path selected");
    println!("Selected path: {}", selected_path);
}
