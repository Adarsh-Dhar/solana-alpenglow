use std::env;
use alpenglow_formal::certificate;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let mut validators = 4;
    let mut slots = 3;
    let mut adversaries = 1;
    let mut test_type = "formal";
    
    for i in 0..args.len() {
        if args[i] == "--validators" && i + 1 < args.len() {
            validators = args[i + 1].parse().unwrap_or(4);
        } else if args[i] == "--slots" && i + 1 < args.len() {
            slots = args[i + 1].parse().unwrap_or(3);
        } else if args[i] == "--adversaries" && i + 1 < args.len() {
            adversaries = args[i + 1].parse().unwrap_or(1);
        } else if args[i] == "--test-type" && i + 1 < args.len() {
            test_type = &args[i + 1];
        }
    }
    
    println!("Running certificate formal verification: {} test, {} validators ({} adversarial), {} slots", 
             test_type, validators, adversaries, slots);
    
    match test_type {
        "formal" => {
            certificate::run_formal_verification();
            println!("Certificate formal verification completed");
        },
        "test" => {
            certificate::test_certificate_model(validators, slots, adversaries);
            println!("Certificate model test completed");
        },
        _ => {
            println!("Unknown test type: {}", test_type);
            std::process::exit(1);
        }
    }
}
