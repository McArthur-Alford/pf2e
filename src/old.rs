type UpdateId = usize;

enum Filter<T> {
    Pass,                       // Preserves the lower layer
    New(T),                     // Overwrites the lower layer
    Mod(Box<dyn Fn(T) -> T>)    // Modifies the lower layer
}

struct Lense<T> {
    update_id: UpdateId,
    filter: Filter<T>,
    cache: Option<T>,
}

enum FieldType<T> {
    Constant(T),
    Derivative(Box<dyn Fn(&State, UpdateId) -> T>),
}

struct Field<T> where T: Copy + Clone + Eq + PartialEq {
    value: FieldType<T>,
    lenses: Vec<Lense<T>>,
}

impl<T> Field<T> where T: Copy + Clone + Eq + PartialEq {
    fn new_const(t: T) -> Self {
        Field {
            value: FieldType::Constant(t),
            lenses: Vec::new(),
        }
    }

    fn new_dyn(f: Box<dyn Fn(&State, UpdateId) -> T>) -> Self {
        Field {
            value: FieldType::Derivative(f),
            lenses: Vec::new()
        }
    }

    fn at(&self, update_id: usize, state: &State) -> T {
        // Returns the value of the field, applying all lenses up to the given update_id
        let mut value = match &self.value {
            FieldType::Constant(t) => *t,
            FieldType::Derivative(f) => f(state, update_id),
        };
        for lense in self.lenses.iter() {
            if lense.update_id <= update_id {
                if let Some(cache) = lense.cache {
                    value = cache;
                } else {
                    match &lense.filter {
                        Filter::Pass => (),
                        Filter::New(t) => value = *t,
                        Filter::Mod(f) => value = f(self.at(update_id-1, state)),
                    };
                }
            } else {
                break;
            };
        }
        value
    }

    fn base(&self, state: &State) -> T {
        // Returns the base value
        match &self.value {
            FieldType::Constant(t) => *t,
            FieldType::Derivative(f) => f(state, 0),
        }
    }
}

struct Update {
    id: UpdateId,
    apply: Box<dyn Fn(&State) -> State>, // Maps the old state to a new state with the update applied
}

struct Meta {
    // Stores meta state inforation
    updates: Vec<Update>,
    current_id: UpdateId,
}

impl Default for Meta {
    fn default() -> Self {
        Meta {
            updates: Vec::new(),
            current_id: 0
        }
    }
}

struct Base {
    strength: Field<u8>,
    dexterity: Field<u8>,
    constitution: Field<u8>,
    intelligence: Field<u8>,
    wisdom: Field<u8>,
    charisma: Field<u8>
}

impl Default for Base {
    fn default() -> Self {
        Base {
            strength: Field::new_const(10),
            dexterity: Field::new_const(10),
            constitution: Field::new_const(10),
            intelligence: Field::new_const(10),
            wisdom: Field::new_const(10),
            charisma: Field::new_const(10),
        }
    }
}

struct State {
    meta: Meta,
    base: Base,
}

impl State {
    fn new() -> Self {
        State {
            meta: Meta::default(),
            base: Base::default(),
        }
    }

    fn update(&mut self, update: Update) {
        self.meta.updates.push(update); // Now what?
    }
}

fn main() {
    let mut state = State::new();
    let out = state.base.charisma.at(state.meta.current_id, &state);
    dbg!(out);
}