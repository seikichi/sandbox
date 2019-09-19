mod sexp;

use std::cell::RefCell;
use std::collections::hash_map::HashMap;
use std::fmt;
use std::rc::Rc;

use sexp::*;

enum Value {
    Nil,
    Symbol(String),
    Integer(u64),
    Pair(Box<Value>, Box<Value>),
    PrimitiveProcedure(Box<dyn Fn(&[Rc<Value>]) -> Value>),
    CompoundProcedure {
        body: Vec<Sexp>,
        parameters: Vec<String>,
        environment: Rc<Environment>,
    },
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::Symbol(s) => write!(f, "{}", s),
            Value::Integer(n) => write!(f, "{}", n),
            Value::Pair(car, cdr) => write!(f, "({:?}, {:?})", car, cdr),
            Value::PrimitiveProcedure(_) => write!(f, "[primitive-procedure]"),
            Value::CompoundProcedure { .. } => write!(f, "[compound-procedure]"),
        }
    }
}

#[derive(Debug)]
struct Environment {
    variables: RefCell<HashMap<String, Rc<Value>>>,
    base: Option<Rc<Environment>>,
}

impl Environment {
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

fn eval(exp: &Sexp, env: Rc<Environment>) -> Rc<Value> {
    if let Sexp::Integer(n) = exp {
        return Rc::new(Value::Integer(*n));
    }
    if let Sexp::Symbol(s) = exp {
        return env.lookup(s);
    }
    if let Sexp::Pair(operator, operands) = exp {
        return apply(
            eval(operator, env.clone()),
            &list_of_values(operands, env.clone()),
        );
    }
    panic!("Unknown expression type -- EVAL {:?}", exp);
}

fn list_of_values(exps: &Sexp, env: Rc<Environment>) -> Vec<Rc<Value>> {
    let mut values = vec![];
    let mut p = exps;
    while let Sexp::Pair(car, cdr) = p {
        values.push(eval(car, env.clone()));
        p = cdr;
    }
    values
}

fn apply(procedure: Rc<Value>, arguments: &[Rc<Value>]) -> Rc<Value> {
    match &*procedure {
        Value::PrimitiveProcedure(f) => Rc::new(f(arguments)),
        _ => panic!("Invalid procedure: {:?}", procedure),
    }
}

fn main() {
    let mut variables = HashMap::new();
    variables.insert(
        "+".to_string(),
        Rc::new(Value::PrimitiveProcedure(Box::new(|args| {
            let mut result = 0;
            for i in 0..args.len() {
                if let Value::Integer(n) = &*args[i] {
                    result += *n;
                } else {
                    panic!("Invalid arguments for +: {:?}", args[i]);
                }
            }
            Value::Integer(result)
        }))),
    );

    let env = Rc::new(Environment {
        variables: RefCell::new(variables),
        base: None,
    });

    let source = "(+ 1 (+ 2 3 4))";
    let sexp = parser::expression(source).unwrap();
    println!("{:?}", eval(&sexp, env));
}
