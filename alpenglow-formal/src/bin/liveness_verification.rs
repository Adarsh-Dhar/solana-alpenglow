use std::env;
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
    
    println!("Running liveness formal verification: {} test, {} validators ({} responsive), {} slots", 
             test_type, validators, responsive, slots);
    
    match test_type {
        "formal" => {
            liveness::run_formal_verification();
            println!("Liveness formal verification completed");
        },
        "test" => {
            liveness::test_liveness_model(validators, slots, responsive);
            println!("Liveness model test completed");
        },
        _ => {
            println!("Unknown test type: {}", test_type);
            std::process::exit(1);
        }
    }
}
