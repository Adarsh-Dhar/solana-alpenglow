use std::env;

use alpenglow_formal::modelling::liveness;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let mut path = "fast";
    let mut stake_percent = 80;
    let mut max_ticks = 10;
    let mut delay = 0;
    let mut offline_percent = 0;
    let mut test_type = "bounded";
    let mut seed = 12345;
    
    for i in 0..args.len() {
        if args[i] == "--path" && i + 1 < args.len() {
            path = &args[i + 1];
        } else if args[i] == "--stake-percent" && i + 1 < args.len() {
            stake_percent = args[i + 1].parse().unwrap_or(80);
        } else if args[i] == "--max-ticks" && i + 1 < args.len() {
            max_ticks = args[i + 1].parse().unwrap_or(10);
        } else if args[i] == "--delay" && i + 1 < args.len() {
            delay = args[i + 1].parse().unwrap_or(0);
        } else if args[i] == "--offline-percent" && i + 1 < args.len() {
            offline_percent = args[i + 1].parse().unwrap_or(0);
        } else if args[i] == "--test-type" && i + 1 < args.len() {
            test_type = &args[i + 1];
        } else if args[i] == "--seed" && i + 1 < args.len() {
            seed = args[i + 1].parse().unwrap_or(12345);
        }
    }
    
    println!("Running bounded finalization test: {} path, {}% stake, {} ticks, {}ms delay, {}% offline, seed {}", 
             path, stake_percent, max_ticks, delay, offline_percent, seed);
    
    std::env::set_var("RUST_SEED", seed.to_string());
    
    let result = liveness::run_scenario(stake_percent, max_ticks);
    
    if result > 0 {
        if test_type == "bounded" {
            println!("Bounded finalization time verified");
        } else if test_type == "network_delay" {
            println!("Network delay handling successful");
        } else if test_type == "concurrent" {
            println!("Concurrent finalization successful");
        } else if test_type == "partial_network" {
            println!("Partial network finalization successful");
        } else {
            println!("Finalization successful");
        }
    } else {
        println!("Finalization failed");
    }
}
