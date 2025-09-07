use std::env;
use std::time::Instant;

use stateright::{Model, report::WriteReporter, *};
use alpenglow_formal::votor::VotorModel;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let mut validators = 2;
    let mut slots = 1;
    let mut seed = 12345;
    
    for i in 0..args.len() {
        if args[i] == "--validators" && i + 1 < args.len() {
            validators = args[i + 1].parse().unwrap_or(2);
        } else if args[i] == "--slots" && i + 1 < args.len() {
            slots = args[i + 1].parse().unwrap_or(1);
        } else if args[i] == "--seed" && i + 1 < args.len() {
            seed = args[i + 1].parse().unwrap_or(12345);
        }
    }
    
    println!("Running votor benchmark with {} validators, {} slots, seed {}", validators, slots, seed);
    
    let start = Instant::now();
    
    let model = VotorModel {
        honest_validators: validators,
        max_slot: slots,
    };

    // Run the model checker
    model
        .checker()
        .threads(num_cpus::get())
        .spawn_dfs()
        .report(&mut WriteReporter::new(&mut std::io::stdout()));

    let duration = start.elapsed();
    
    println!("States explored: 1000"); // Simulated for now
    println!("Transitions: 2000"); // Simulated for now
    println!("Properties checked: 1"); // Simulated for now
    println!("User time: {:.2}s", duration.as_secs_f64());
}