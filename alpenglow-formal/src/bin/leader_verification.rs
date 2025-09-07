use std::env;
use alpenglow_formal::leader;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let mut validators = 3;
    let mut slots = 5;
    let mut test_type = "formal";
    
    for i in 0..args.len() {
        if args[i] == "--validators" && i + 1 < args.len() {
            validators = args[i + 1].parse().unwrap_or(3);
        } else if args[i] == "--slots" && i + 1 < args.len() {
            slots = args[i + 1].parse().unwrap_or(5);
        } else if args[i] == "--test-type" && i + 1 < args.len() {
            test_type = &args[i + 1];
        }
    }
    
    println!("Running leader formal verification: {} test, {} validators, {} slots", 
             test_type, validators, slots);
    
    match test_type {
        "formal" => {
            leader::run_formal_verification();
            println!("Leader formal verification completed");
        },
        "test" => {
            leader::test_leader_model(validators, slots);
            println!("Leader model test completed");
        },
        _ => {
            println!("Unknown test type: {}", test_type);
            std::process::exit(1);
        }
    }
}
