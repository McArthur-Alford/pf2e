use std::collections::HashSet;

#[derive(Clone)]
pub(crate) struct Meta {}

#[derive(Clone)]
pub(crate) struct TestBase {
    pub(crate) magics: u8,
    pub(crate) woos: i32,
    pub(crate) name: String,
}

#[derive(Clone)]
pub(crate) struct State<T> {
    pub(crate) meta: Meta,
    pub(crate) base: T,
}

pub(crate) type Filter<T> = Box<dyn Fn(State<T>) -> State<T>>;

#[derive(Eq, PartialEq, Hash)]
pub(crate) enum Tag {}

#[derive(PartialEq)]
pub(crate) enum Resolved {
    // Should represent 3 states:
    // An update is resolved and we should resolve here
    // An update could be considered resolved if necessary, but there is more to be done
    // An update is unresolved
    Resolved,
    Partial,
    Unresolved,
}

pub(crate) struct Update<T> {
    pub(crate) filter: Filter<T>,
    pub(crate) id: usize,
    pub(crate) tags: HashSet<Tag>,
    pub(crate) resolved: Resolved,
}

pub(crate) enum UserInput {}

pub(crate) enum InvalidAction {
    BadPredicate,
}

pub(crate) enum ActionResponse<T> {
    Valid(Vec<Update<T>>),
    Invalid(InvalidAction),
    RequestInput(UserInput),
}

/** Action
 *  An action generates a chain of updates based on the current state.
 *  An action can request user input.
 *  An action requires a predicate to be satisfied by the current state.
 */
pub(crate) struct Action<T> {
    pub(crate) generator: Box<dyn Fn(&State<T>) -> ActionResponse<T>>,
}

impl<T> Action<T> {
    pub(crate) fn new(generator: Box<dyn Fn(&State<T>) -> ActionResponse<T>>) -> Self {
        Self { generator }
    }

    pub(crate) fn apply(&self, state: &State<T>) -> ActionResponse<T> {
        (self.generator)(state)
    }
}

pub(crate) enum RuleResponse<T> {
    Skip,
    Divert(Action<T>), // Kill all future updates in the chain, create a new action
    Revert(Action<T>), // Kill all future and past updates in the chain
    Inject(Action<T>), // Inject a new action into the update chain
    // (This must satisfy the predicate of the next update)
    Attach(Tag), // Attach a tag to the current update (to be used by future rules)
}

pub(crate) type Rule<T> = Box<dyn Fn(&State<T>, &Update<T>) -> RuleResponse<T>>;

pub(crate) struct Engine<T: Clone> {
    // T is the base type (world data)
    pub(crate) action: Option<Action<T>>,      // Only 1 action can be active at a time
    pub(crate) rules: Vec<Rule<T>>,            // Rules are applied to each update in the chain
    pub(crate) updates: Vec<Update<T>>,        // Updates are applied to the base type
    pub(crate) update: usize,                  // The current update in the chain
    pub(crate) state: State<T>,                // The current state of the engine
}

impl<T: Clone> Engine<T> {
    pub(crate) fn step(&mut self) {
        // Handle Action
        if self.process_action() {
            return; // We don't want to process rules if we have an action
        }

        // Handle Rules
        if self.process_rule() {
            return; // We don't want to process updates if we have a rule
        }

        // // Handle Updates
        // if self.process_update() {
        //     return; // We are done for this step
        // }
    }

    fn process_rule(&mut self) -> bool {
        if let Some(update) = self.updates.last_mut() {
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
                        self.action = Some(a);
                        return true;
                    }
                    RuleResponse::Revert(a) => {
                        // Kill all future and past updates in the chain
                        self.updates.clear();
                        self.update = 0;
                        self.action = Some(a);
                        return true;
                    }
                    RuleResponse::Inject(a) => {
                        // Inject a new action into the update chain
                        self.action = Some(a);
                        return true;
                    }
                    RuleResponse::Attach(t) => {
                        // Attach a tag to the current update (to be used by future rules)
                        update.tags.insert(t);
                    }
                }
            }
            self.process_update();
            self.update += 1;
            true
        } else {
            false
        }
    }

    fn process_action(&mut self) -> bool {
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
            true
        } else {
            false
        }
    }

    fn process_update(&mut self) -> bool {
        if let Some(update) = self.updates.get_mut(self.update) {
            if update.resolved == Resolved::Resolved {
                // This update represents a FULLY resolved state
                // We can apply it to the state
                if let Some(state) = self.resolve() {
                    self.state = state;
                }
            }
            true
        } else {
            false
        }
    }

    /**
     * 1. Check if the current update is resolved
     * 2. If it is, apply all updates in the chain to the state permenantly
     * 3. If it is not, return None
     * 4. If successful, clean up the update chain in case we resolved early
     */
    pub(crate) fn resolve(&mut self) -> Option<State<T>> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut engine = Engine {
            action: None,
            rules: vec![],
            updates: vec![],
            update: 0,
            state: State {
                meta: Meta {},
                base: TestBase {
                    magics: 0,
                    woos: 0,
                    name: String::from(""),
                },
            },
        };
        engine.step();
        assert_eq!(engine.update, 0);
    }

    #[test]
    fn test2() {
        let mut engine = Engine {
            action: None,
            rules: vec![],
            updates: vec![],
            update: 0,
            state: State {
                meta: Meta {},
                base: TestBase {
                    magics: 0,
                    woos: 0,
                    name: String::from(""),
                },
            },
        };
        engine.updates.push(Update {
            filter: Box::new(|state| {
                let mut state = state.clone();
                state.base.magics += 1;
                state
            }),
            resolved: Resolved::Resolved,
            tags: HashSet::new(),
            id: engine.update + 1,
        });
        engine.step();
        assert_eq!(engine.update, 1);
        assert_eq!(engine.state.base.magics, 1);
    }
}