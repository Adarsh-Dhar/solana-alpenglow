use alpenglow_formal::votor::VotorModel;
use stateright::{report::WriteReporter, *};
use std::env;

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
    
    println!("Running safety verification with {} validators, {} slots, seed {}", validators, slots, seed);
    
    let model = VotorModel {
        honest_validators: validators,
        max_slot: slots,
    };

    let result = model
        .checker()
        .threads(num_cpus::get())
        .spawn_dfs()
        .report(&mut WriteReporter::new(&mut std::io::stdout()));
    
    // Check if safety property was verified
    if result.discoveries().is_empty() {
        println!("Property 'safety' is always true");
    } else {
        println!("Property 'safety' has counterexamples");
    }
}