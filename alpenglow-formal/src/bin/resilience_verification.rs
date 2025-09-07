use std::env;
use alpenglow_formal::modelling::resilience;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let mut validators = 4;
    let mut slots = 3;
    let mut byzantine = 1;
    let mut test_type = "formal";
    
    for i in 0..args.len() {
        if args[i] == "--validators" && i + 1 < args.len() {
            validators = args[i + 1].parse().unwrap_or(4);
        } else if args[i] == "--slots" && i + 1 < args.len() {
            slots = args[i + 1].parse().unwrap_or(3);
        } else if args[i] == "--byzantine" && i + 1 < args.len() {
            byzantine = args[i + 1].parse().unwrap_or(1);
        } else if args[i] == "--test-type" && i + 1 < args.len() {
            test_type = &args[i + 1];
        }
    }
    
    println!("Running resilience formal verification: {} test, {} validators ({} Byzantine), {} slots", 
             test_type, validators, byzantine, slots);
    
    match test_type {
        "formal" => {
            resilience::run_formal_verification();
            println!("Resilience formal verification completed");
        },
        "test" => {
            resilience::test_resilience_model(validators, slots, byzantine);
            println!("Resilience model test completed");
        },
        _ => {
            println!("Unknown test type: {}", test_type);
            std::process::exit(1);
        }
    }
}
