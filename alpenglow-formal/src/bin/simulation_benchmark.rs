use std::env;
use std::time::Instant;

use alpenglow_formal::modelling::liveness;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let mut validators = 10;
    let mut simulations = 1000;
    let mut seed = 12345;
    
    for i in 0..args.len() {
        if args[i] == "--validators" && i + 1 < args.len() {
            validators = args[i + 1].parse().unwrap_or(10);
        } else if args[i] == "--simulations" && i + 1 < args.len() {
            simulations = args[i + 1].parse().unwrap_or(1000);
        } else if args[i] == "--seed" && i + 1 < args.len() {
            seed = args[i + 1].parse().unwrap_or(12345);
        }
    }
    
    println!("Running simulation benchmark with {} validators, {} simulations, seed {}", validators, simulations, seed);
    
    let start = Instant::now();
    
    let mut successful_simulations = 0;
    
    for i in 0..simulations {
        let result = liveness::run_scenario(80, 10); // 80% responsive stake
        if result > 0 {
            successful_simulations += 1;
        }
    }
    
    let duration = start.elapsed();
    
    println!("Simulations completed: {}", simulations);
    println!("Successful simulations: {}", successful_simulations);
    println!("Success rate: {:.2}%", (successful_simulations as f64 / simulations as f64) * 100.0);
    println!("User time: {:.2}s", duration.as_secs_f64());
}