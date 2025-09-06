use stateright::{report::WriteReporter, *};

// 1. DEFINE THE ACTIONS that can happen.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum Action {
    Increment,
    Decrement,
}

// 2. DEFINE THE STATE of the system.
#[derive(Clone, Debug, Default, Hash, PartialEq)]
struct State {
    counter: i32,
}

impl State {
    fn new() -> Self {
        Self { counter: 0 }
    }
}

// 3. IMPLEMENT THE MODEL LOGIC.
impl Model for State {
    type State = State;
    type Action = Action;

    fn init_states(&self) -> Vec<Self::State> {
        vec![State::new()]
    }

    fn actions(&self, _state: &Self::State, actions: &mut Vec<Self::Action>) {
        // Always allow both increment and decrement actions
        actions.push(Action::Increment);
        actions.push(Action::Decrement);
    }

    fn next_state(&self, last_state: &Self::State, action: Self::Action) -> Option<Self::State> {
        let mut state = last_state.clone();
        match action {
            Action::Increment => {
                state.counter += 1;
            }
            Action::Decrement => {
                state.counter -= 1;
            }
        }
        Some(state)
    }

    fn properties(&self) -> Vec<Property<Self>> {
        vec![
            Property::<Self>::always("counter <= 5", |_, state| {
                state.counter <= 5
            }),
            Property::<Self>::always("counter >= -5", |_, state| {
                state.counter >= -5
            }),
        ]
    }
}

// 4. CONFIGURE AND RUN THE CHECKER.
fn main() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    println!("Model checking counter system...");

    State::new()
        .checker()
        .threads(num_cpus::get())
        .spawn_dfs()
        .report(&mut WriteReporter::new(&mut std::io::stdout()));
}