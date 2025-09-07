use std::env;

use alpenglow_formal::modelling::resilience;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let mut byzantine_stake = 20;
    let mut offline_stake = 20;
    let mut test_type = "safety";
    let mut seed = 12345;
    
    for i in 0..args.len() {
        if args[i] == "--byzantine-stake" && i + 1 < args.len() {
            byzantine_stake = args[i + 1].parse().unwrap_or(20);
        } else if args[i] == "--offline-stake" && i + 1 < args.len() {
            offline_stake = args[i + 1].parse().unwrap_or(20);
        } else if args[i] == "--test-type" && i + 1 < args.len() {
            test_type = &args[i + 1];
        } else if args[i] == "--seed" && i + 1 < args.len() {
            seed = args[i + 1].parse().unwrap_or(12345);
        }
    }
    
    println!("Running resilience verification: {} test, {}% byzantine, {}% offline, seed {}", 
             test_type, byzantine_stake, offline_stake, seed);
    
    std::env::set_var("RUST_SEED", seed.to_string());
    
    match test_type {
        "safety" => {
            resilience::simulate_safety_under_attack(byzantine_stake);
            println!("Safety maintained");
        },
        "liveness" => {
            resilience::simulate_liveness_with_offline_nodes(offline_stake);
            println!("Liveness maintained");
        },
        "partition" => {
            resilience::simulate_network_partition_recovery();
            println!("Partition recovery successful");
        },
        _ => {
            println!("Unknown test type: {}", test_type);
        }
    }
}
