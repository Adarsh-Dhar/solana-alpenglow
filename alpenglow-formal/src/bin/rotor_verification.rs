use std::env;
use alpenglow_formal::rotor;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let mut nodes = 4;
    let mut slots = 3;
    let mut test_type = "formal";
    
    for i in 0..args.len() {
        if args[i] == "--nodes" && i + 1 < args.len() {
            nodes = args[i + 1].parse().unwrap_or(4);
        } else if args[i] == "--slots" && i + 1 < args.len() {
            slots = args[i + 1].parse().unwrap_or(3);
        } else if args[i] == "--test-type" && i + 1 < args.len() {
            test_type = &args[i + 1];
        }
    }
    
    println!("Running rotor formal verification: {} test, {} nodes, {} slots", 
             test_type, nodes, slots);
    
    match test_type {
        "formal" => {
            rotor::run_formal_verification();
            println!("Rotor formal verification completed");
        },
        "test" => {
            rotor::test_rotor_model(nodes, slots);
            println!("Rotor model test completed");
        },
        _ => {
            println!("Unknown test type: {}", test_type);
            std::process::exit(1);
        }
    }
}
