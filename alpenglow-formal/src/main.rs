mod votor;

use stateright::{report::WriteReporter, *};
use votor::VotorModel;

fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    println!("Model checking Votor consensus system...");
    println!("This model verifies the safety of the dual-path finality mechanism:");
    println!("- Fast Path: Finalization in one round with >= 80% stake");
    println!("- Slow Path: Finalization in two rounds with >= 60% stake each");
    println!();

    let model = VotorModel {
        honest_validators: 3,
        max_slot: 2, // Check up to slot 2
    };

    model
        .checker()
        .threads(num_cpus::get())
        .spawn_dfs()
        .report(&mut WriteReporter::new(&mut std::io::stdout()));
}