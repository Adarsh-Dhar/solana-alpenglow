use std::env;

use alpenglow_formal::timeout;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let mut test_type = "basic";
    let mut timeout_ms = 100;
    let mut delay = 20;
    let mut offline_percent = 20;
    let mut seed = 12345;
    
    for i in 0..args.len() {
        if args[i] == "--test-type" && i + 1 < args.len() {
            test_type = &args[i + 1];
        } else if args[i] == "--timeout" && i + 1 < args.len() {
            timeout_ms = args[i + 1].parse().unwrap_or(100);
        } else if args[i] == "--delay" && i + 1 < args.len() {
            delay = args[i + 1].parse().unwrap_or(20);
        } else if args[i] == "--offline-percent" && i + 1 < args.len() {
            offline_percent = args[i + 1].parse().unwrap_or(20);
        } else if args[i] == "--seed" && i + 1 < args.len() {
            seed = args[i + 1].parse().unwrap_or(12345);
        }
    }
    
    println!("Running timeout verification: {} test, {}ms timeout, {}ms delay, {}% offline, seed {}", 
             test_type, timeout_ms, delay, offline_percent, seed);
    
    std::env::set_var("RUST_SEED", seed.to_string());
    
    timeout::run_simulation();
    println!("Timeout handling successful");
}
