use std::collections::HashSet;

#[derive(Clone, Copy)]
struct Meta {}

struct TestBase {
    magics: u8,
    woos: i32,
    name: String,
}

#[derive(Clone, Copy)]
struct State<T> {
    meta: Meta,
    base: T,
}

type Filter<T> = Box<dyn Fn(State<T>) -> State<T>>;

#[derive(Eq, PartialEq, Hash)]
enum Tag {}

#[derive(PartialEq)]
enum Resolved {
    // Should represent 3 states:
    // An update is resolved and we should resolve here
    // An update could be considered resolved if necessary, but there is more to be done
    // An update is unresolved
    Resolved,
    Partial,
    Unresolved,
}

struct Update<T> {
    filter: Filter<T>,
    id: usize,
    tags: HashSet<Tag>,
    next: Option<Box<Update<T>>>,
    resolved: Resolved,
}

enum UserInput {}

enum InvalidAction {
    BadPredicate,
}

enum ActionResponse<T> {
    Valid(Vec<Update<T>>),
    Invalid(InvalidAction),
    RequestInput(UserInput),
}

/** Action
 *  An action generates a chain of updates based on the current state.
 *  An action can request user input.
 *  An action requires a predicate to be satisfied by the current state.
 */
struct Action<T> {
    generator: Box<dyn Fn(&State<T>) -> ActionResponse<T>>,
}

impl<T> Action<T> {
    fn new(generator: Box<dyn Fn(&State<T>) -> ActionResponse<T>>) -> Self {
        Self { generator }
    }

    fn apply(&self, state: &State<T>) -> ActionResponse<T> {
        (self.generator)(state)
    }
}

enum RuleResponse<T> {
    Skip,
    Divert(Action<T>), // Kill all future updates in the chain, create a new action
    Revert(Action<T>), // Kill all future and past updates in the chain
    Inject(Action<T>), // Inject a new action into the update chain
    // (This must satisfy the predicate of the next update)
    Attach(Tag), // Attach a tag to the current update (to be used by future rules)
}
type Rule<T> = Box<dyn Fn(&State<T>, &Update<T>) -> RuleResponse<T>>;

struct Engine<T: Copy + Clone> {
    // T is the base type (world data)
    action: Option<Action<T>>,      // Only 1 action can be active at a time
    rules: Vec<Rule<T>>,            // Rules are applied to each update in the chain
    updates: Vec<Update<T>>,        // Updates are applied to the base type
    update: usize,                  // The current update in the chain
    state: State<T>,                // The current state of the engine
}

impl<T: Copy + Clone> Engine<T> {
    fn step(&mut self) {
        // Handle Action
        if let Some(action) = &self.action {
            let response = action.apply(&self.state);
            match response {
                ActionResponse::Valid(updates) => {
                    todo!();
                }
                ActionResponse::Invalid(_) => {
                    todo!();
                }
                ActionResponse::RequestInput(_) => {
                    todo!();
                }
            }
        }

        // Handle Rules
        let update = self.updates.get_mut(self.update).unwrap();
        // Update is moved into the for loop
        for rule in self.rules.iter() {
            let response = 
                rule(&self.state, &update);
            
            match response {
                RuleResponse::Skip => {
                    // Do nothing
                }
                RuleResponse::Divert(a) => {
                    // Kill all future updates in the chain, create a new action
                    self.updates.truncate(self.update);
                    self.action = a;
                    break;
                }
                RuleResponse::Revert(a) => {
                    // Kill all future and past updates in the chain
                    self.updates.clear();
                    self.update = 0;
                    self.action = a;
                    break;
                }
                RuleResponse::Inject(a) => {
                    // Inject a new action into the update chain
                    self.action = a;
                    break;
                }
                RuleResponse::Attach(t) => {
                    // Attach a tag to the current update (to be used by future rules)
                    update.tags.insert(t);
                }
            }
        }
        self.update += 1;

        // Handle Updates
    }

    /**
     * 1. Check if the current update is resolved
     * 2. If it is, apply all updates in the chain to the state permenantly
     * 3. If it is not, return None
     * 4. If successful, clean up the update chain in case we resolved early
     */
    fn resolve(&mut self) -> Option<State<T>> {
        let update = self.updates.last().unwrap();
        if update.resolved == Resolved::Unresolved {
            return None;
        }
        let mut state = self.state.clone();
        for update in self.updates.iter() {
            let filter = &update.filter;
            state = filter(state);
        }
        self.updates.clear();
        Some(state)
    }
}

fn main() {}
