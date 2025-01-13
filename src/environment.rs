use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::token::{LiteralType, Token};

pub struct Environment {
    values: HashMap<String, LiteralType>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

pub enum EnvironmentError {
    AssignError,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn with_enclosing(enclosing: &Rc<RefCell<Environment>>) -> Self {
        Self {
            values: HashMap::new(),
            enclosing: Some(Rc::clone(enclosing)),
        }
    }

    pub fn define(&mut self, name: &str, val: LiteralType) {
        // do not like this at all. String is allocated each time a variable is defined. Might be
        // bad or might be good. I don't know :D
        self.values.insert(name.to_string(), val);
    }

    pub fn assign(&mut self, name: &Token, val: LiteralType) -> Result<(), EnvironmentError> {
        let cloned = val.clone();
        let assigned = self
            .values
            .get_mut(&name.lexeme)
            .map(|l| *l = val)
            .ok_or(EnvironmentError::AssignError);

        if assigned.is_err() {
            if let Some(enclosing) = &mut self.enclosing {
                return enclosing.borrow_mut().assign(name, cloned);
            }
        }

        assigned
    }

    pub fn get(&self, name: &Token) -> Option<LiteralType> {
        //self.values.get(&name.lexeme).cloned()

        let value = self.values.get(&name.lexeme);

        if value.is_none() {
            if let Some(enclosing) = &self.enclosing {
                return enclosing.borrow().get(name);
            }
        }

        value.cloned()
    }
}
