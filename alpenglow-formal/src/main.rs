use stateright::{Actor, ActorModel, Checker, Expectation, Model};
use std::sync::Arc;

// 1. DEFINE THE STATE of a single actor (a simple counter).
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct Counter {
    value: u32,
}

// 2. DEFINE THE ACTIONS that can happen.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum Action {
    Increment,
    Decrement,
}

// 3. IMPLEMENT THE ACTOR LOGIC.
impl Actor for Counter {
    type Msg = (); // No messages needed for this simple model.
    type State = u32; // The actor's state is just a number.
    type Timer = (); // No timers needed.

    fn on_start(&self, _id: stateright::ActorId, o: &mut stateright::Out<Self>) {
        // When the actor starts, it can choose to either increment or decrement.
        o.send_action(Action::Increment);
        o.send_action(Action::Decrement);
    }
    
    fn on_action(&self, _id: stateright::ActorId, state: &mut Self::State, action: Self::Action, o: &mut stateright::Out<Self>) {
        // Based on the action, change the state.
        match action {
            Action::Increment => *state += 1,
            Action::Decrement => *state -= 1,
        }
        // After acting, the actor can choose its next actions.
        o.send_action(Action::Increment);
        o.send_action(Action::Decrement);
    }
}

// 4. CONFIGURE AND RUN THE CHECKER.
fn main() {
    // Define the model. We'll have two counter actors.
    let model = ActorModel::new(0, // Initial global state
        vec![
            Counter { value: 0 },
            Counter { value: 0 },
        ])
        .property(Expectation::Always, "value <= 5", |_, state| {
            // This property checks that the sum of counters never exceeds 5.
            state.actor_states.iter().all(|s| *s <= 5)
        });

    // Run the checker.
    Checker::new(model).run_mc();
}