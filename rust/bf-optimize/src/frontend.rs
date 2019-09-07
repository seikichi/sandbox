#[derive(Debug, PartialEq, Eq)]
pub enum Expr {
    Add(u8),
    Sub(u8),
    Right(usize),
    Left(usize),
    Out,
    In,
    Loop(Vec<Expr>),
    Clear,
    ScanRight,
    ScanLeft,
    AddMul(usize, u8),
    SubMul(usize, u8),
    Block(Vec<Expr>),
}

pub mod parser {
    include!(concat!(env!("OUT_DIR"), "/grammar.rs"));
}
