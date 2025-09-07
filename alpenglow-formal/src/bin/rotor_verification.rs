use std::env;

use alpenglow_formal::rotor;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let mut test_type = "basic";
    let mut nodes = 20;
    let mut fanout = 3;
    let mut fault_percent = 20;
    let mut topology = "mesh";
    let mut seed = 12345;
    
    for i in 0..args.len() {
        if args[i] == "--test-type" && i + 1 < args.len() {
            test_type = &args[i + 1];
        } else if args[i] == "--nodes" && i + 1 < args.len() {
            nodes = args[i + 1].parse().unwrap_or(20);
        } else if args[i] == "--fanout" && i + 1 < args.len() {
            fanout = args[i + 1].parse().unwrap_or(3);
        } else if args[i] == "--fault-percent" && i + 1 < args.len() {
            fault_percent = args[i + 1].parse().unwrap_or(20);
        } else if args[i] == "--topology" && i + 1 < args.len() {
            topology = &args[i + 1];
        } else if args[i] == "--seed" && i + 1 < args.len() {
            seed = args[i + 1].parse().unwrap_or(12345);
        }
    }
    
    println!("Running rotor verification: {} test, {} nodes, fanout {}, {}% fault, {} topology, seed {}", 
             test_type, nodes, fanout, fault_percent, topology, seed);
    
    std::env::set_var("RUST_SEED", seed.to_string());
    
    rotor::run_simulation();
    println!("Rotor sampling successful");
}
