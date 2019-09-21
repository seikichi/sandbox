use super::value::*;

use std::cell::RefCell;
use std::collections::hash_map::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct Environment {
    variables: RefCell<HashMap<String, Rc<Value>>>,
    base: Option<Rc<Environment>>,
}

impl Environment {
    pub fn empty() -> Rc<Self> {
        Rc::new(Environment {
            variables: RefCell::new(HashMap::new()),
            base: None,
        })
    }

    pub fn define(&self, var: &str, value: Rc<Value>) {
        let mut variables = self.variables.borrow_mut();
        variables.insert(var.to_string(), value);
    }

    pub fn set(&self, var: &str, value: Rc<Value>) {
        let mut variables = self.variables.borrow_mut();
        if variables.contains_key(var) {
            variables.insert(var.to_string(), value);
        } else if let Some(base) = &self.base {
            base.set(var, value);
        } else {
            panic!("Unbound variable: {}", var);
        }
    }

    pub fn extend(vars: &[String], vals: &[Rc<Value>], base: &Rc<Environment>) -> Rc<Self> {
        if vars.len() < vals.len() {
            panic!("Too many arguments supplied");
        }
        if vars.len() > vals.len() {
            panic!("Too few arguments supplied");
        }

        let mut variables = HashMap::new();
        for (var, val) in vars.iter().zip(vals.iter()) {
            variables.insert(var.clone(), val.clone());
        }

        Rc::new(Self {
            variables: RefCell::new(variables),
            base: Some(base.clone()),
        })
    }

    pub fn lookup(&self, var: &str) -> Rc<Value> {
        let variables = self.variables.borrow();
        if let Some(value) = variables.get(var) {
            return value.clone();
        }
        if let Some(base) = &self.base {
            return base.lookup(var);
        }
        panic!("Unbound variable: {}", var);
    }
}
