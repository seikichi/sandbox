#[derive(Debug, PartialEq, Eq)]
pub enum Expr {
    Add(u8),
    Sub(u8),
    Right(u8),
    Left(u8),
    Out,
    In,
    Loop(Vec<Expr>),
    Clear,
}

pub mod parser {
    include!(concat!(env!("OUT_DIR"), "/grammar.rs"));
}
