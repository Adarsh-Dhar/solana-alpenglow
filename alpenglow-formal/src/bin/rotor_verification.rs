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
    
    match test_type {
        "basic" => {
            rotor::run_simulation();
            println!("Rotor sampling successful");
        },
        "stake_weighted" => {
            rotor::test_stake_weighted_selection(nodes);
            println!("Stake-weighted selection successful");
        },
        "fanout" => {
            rotor::test_fanout_optimization(fanout);
            println!("Fanout optimization successful");
        },
        "dissemination" => {
            rotor::test_message_dissemination(nodes);
            println!("Message dissemination successful");
        },
        "topology" => {
            rotor::test_topology_adaptation(topology);
            println!("Topology adaptation successful");
        },
        "fault_tolerance" => {
            rotor::test_fault_tolerance(fault_percent);
            println!("Fault tolerance successful");
        },
        "load_balancing" => {
            rotor::test_load_balancing();
            println!("Load balancing successful");
        },
        "scalability" => {
            rotor::test_scalability(nodes);
            println!("Scalability test successful");
        },
        _ => {
            println!("Unknown test type: {}", test_type);
            std::process::exit(1);
        }
    }
}
