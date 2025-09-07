use std::env;

use alpenglow_formal::modelling::liveness;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let mut responsive_stake = 80;
    let mut seed = 12345;
    
    for i in 0..args.len() {
        if args[i] == "--responsive-stake" && i + 1 < args.len() {
            responsive_stake = args[i + 1].parse().unwrap_or(80);
        } else if args[i] == "--seed" && i + 1 < args.len() {
            seed = args[i + 1].parse().unwrap_or(12345);
        }
    }
    
    println!("Running liveness verification with {}% responsive stake, seed {}", responsive_stake, seed);
    
    std::env::set_var("RUST_SEED", seed.to_string());
    
    let result = liveness::run_scenario(responsive_stake, 10);
    
    if result > 0 {
        println!("Liveness Success! Slot finalized at T={}.", result);
    } else {
        println!("Liveness Failure! Slot did not finalize within the time limit.");
    }
}
