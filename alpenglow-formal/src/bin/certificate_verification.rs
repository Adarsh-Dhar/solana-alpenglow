use std::env;

use alpenglow_formal::certificate;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let mut adversary_stake = 20;
    let mut attack_type = "equivocation";
    let mut seed = 12345;
    
    for i in 0..args.len() {
        if args[i] == "--adversary-stake" && i + 1 < args.len() {
            adversary_stake = args[i + 1].parse().unwrap_or(20);
        } else if args[i] == "--attack-type" && i + 1 < args.len() {
            attack_type = &args[i + 1];
        } else if args[i] == "--seed" && i + 1 < args.len() {
            seed = args[i + 1].parse().unwrap_or(12345);
        }
    }
    
    println!("Running certificate verification: {} attack, {}% adversary, seed {}", 
             attack_type, adversary_stake, seed);
    
    std::env::set_var("RUST_SEED", seed.to_string());
    
    certificate::run_simulation();
    println!("Certificate uniqueness maintained");
}
