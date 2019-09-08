#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expr {
    Add(i32),
    Sub(i32),
    Right(usize),
    Left(usize),
    Out,
    In,
    Loop(Vec<Expr>),
    Clear,
}

pub mod parser {
    include!(concat!(env!("OUT_DIR"), "/grammar.rs"));
}
