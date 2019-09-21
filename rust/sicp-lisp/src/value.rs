use super::environment::*;
use super::sexp::*;
use std::fmt;
use std::rc::Rc;

pub enum Value {
    Nil,
    Symbol(String),
    Integer(i64),
    Pair(Rc<Value>, Rc<Value>),
    PrimitiveProcedure(Box<dyn Fn(&[Rc<Value>]) -> Rc<Value>>),
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
            Value::Pair(car, cdr) => write!(f, "({:?} . {:?})", car, cdr),
            Value::PrimitiveProcedure(_) => write!(f, "#<primitive procedure>"),
            Value::CompoundProcedure { .. } => write!(f, "#<procedure>"),
        }
    }
}

impl Value {
    pub fn ok() -> Rc<Self> {
        Rc::new(Value::Symbol("ok".to_string()))
    }

    pub fn t() -> Rc<Self> {
        Rc::new(Value::Symbol("true".to_string()))
    }

    pub fn f() -> Rc<Self> {
        Rc::new(Value::Symbol("false".to_string()))
    }
}
