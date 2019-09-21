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

    pub fn new() -> Rc<Self> {
        let env = Self::empty();

        env.define_primitive_procedure("+", box |args| {
            let mut result = 0;
            for arg in args {
                if let Value::Integer(n) = &**arg {
                    result += *n;
                } else {
                    panic!("Invalid arguments for +: {:?}", arg);
                }
            }
            Rc::new(Value::Integer(result))
        });

        env.define_primitive_procedure("-", box |args| {
            let mut result = match &*args[0] {
                Value::Integer(n) => *n,
                _ => panic!("Invalid arguments for -: {:?}", args),
            };
            for arg in &args[1..] {
                if let Value::Integer(n) = &**arg {
                    result -= *n;
                } else {
                    panic!("Invalid arguments for -: {:?}", arg);
                }
            }
            Rc::new(Value::Integer(result))
        });

        env.define_primitive_procedure("*", box |args| {
            let mut result = match &*args[0] {
                Value::Integer(n) => *n,
                _ => panic!("Invalid arguments for *: {:?}", args),
            };
            for arg in &args[1..] {
                if let Value::Integer(n) = &**arg {
                    result *= *n;
                } else {
                    panic!("Invalid arguments for *: {:?}", arg);
                }
            }
            Rc::new(Value::Integer(result))
        });

        env.define_primitive_procedure("/", box |args| {
            let mut result = match &*args[0] {
                Value::Integer(n) => *n,
                _ => panic!("Invalid arguments for /: {:?}", args),
            };
            for arg in &args[1..] {
                if let Value::Integer(n) = &**arg {
                    result /= *n;
                } else {
                    panic!("Invalid arguments for /: {:?}", arg);
                }
            }
            Rc::new(Value::Integer(result))
        });

        env.define_primitive_procedure("cons", box |args| {
            if args.len() != 2 {
                panic!("Invalid arguments for cons: {:?}", args);
            }
            Rc::new(Value::Pair(args[0].clone(), args[1].clone()))
        });

        env.define_primitive_procedure("car", box |args| {
            if let Value::Pair(car, _) = &*args[0] {
                return car.clone();
            }
            panic!("Invalid arguments for car: {:?}", args);
        });

        env.define_primitive_procedure("cdr", box |args| {
            if let Value::Pair(_, cdr) = &*args[0] {
                return cdr.clone();
            }
            panic!("Invalid arguments for cdr: {:?}", args);
        });

        env.define_primitive_procedure("display", box |args| {
            if args.len() != 1 {
                panic!("Invalid arguments for display: {:?}", args);
            }
            println!("{:?}", args[0]);
            Rc::new(Value::Nil)
        });

        env
    }

    fn define_primitive_procedure(&self, var: &str, p: Box<dyn Fn(&[Rc<Value>]) -> Rc<Value>>) {
        self.define(var, Rc::new(Value::PrimitiveProcedure(p)));
    }
}
