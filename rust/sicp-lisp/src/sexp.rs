#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Sexp {
    Nil,
    Integer(i64),
    Symbol(String),
    Pair(Box<Sexp>, Box<Sexp>),
}

pub mod parser {
    include!(concat!(env!("OUT_DIR"), "/grammar.rs"));
}
