use std::env;
use std::time::Instant;

use alpenglow_formal::modelling::liveness;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let mut validators = 4;
    let mut slots = 3;
    let mut responsive = 3;
    let mut test_type = "formal";
    
    for i in 0..args.len() {
        if args[i] == "--validators" && i + 1 < args.len() {
            validators = args[i + 1].parse().unwrap_or(4);
        } else if args[i] == "--slots" && i + 1 < args.len() {
            slots = args[i + 1].parse().unwrap_or(3);
        } else if args[i] == "--responsive" && i + 1 < args.len() {
            responsive = args[i + 1].parse().unwrap_or(3);
        } else if args[i] == "--test-type" && i + 1 < args.len() {
            test_type = &args[i + 1];
        }
    }
    
    println!("Running formal verification benchmark: {} test, {} validators ({} responsive), {} slots", 
             test_type, validators, responsive, slots);
    
    let start = Instant::now();
    
    match test_type {
        "formal" => {
            liveness::run_formal_verification();
        },
        "test" => {
            liveness::test_liveness_model(validators, slots, responsive);
        },
        _ => {
            println!("Unknown test type: {}", test_type);
            std::process::exit(1);
        }
    }
    
    let duration = start.elapsed();
    
    println!("Formal verification completed in {:.2}s", duration.as_secs_f64());
}