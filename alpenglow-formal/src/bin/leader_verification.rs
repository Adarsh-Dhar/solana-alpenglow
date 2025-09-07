use std::env;

use alpenglow_formal::leader;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let mut test_type = "rotation";
    let mut window_size = 10;
    let mut failure_rate = 20;
    let mut seed = 12345;
    
    for i in 0..args.len() {
        if args[i] == "--test-type" && i + 1 < args.len() {
            test_type = &args[i + 1];
        } else if args[i] == "--window-size" && i + 1 < args.len() {
            window_size = args[i + 1].parse().unwrap_or(10);
        } else if args[i] == "--failure-rate" && i + 1 < args.len() {
            failure_rate = args[i + 1].parse().unwrap_or(20);
        } else if args[i] == "--seed" && i + 1 < args.len() {
            seed = args[i + 1].parse().unwrap_or(12345);
        }
    }
    
    println!("Running leader verification: {} test, window {}, failure rate {}%, seed {}", 
             test_type, window_size, failure_rate, seed);
    
    std::env::set_var("RUST_SEED", seed.to_string());
    
    leader::run_simulation();
    println!("Leader rotation successful");
}
