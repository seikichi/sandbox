#[derive(Debug, PartialEq, Eq)]
pub enum Expr {
    Add(u8, usize),
    Sub(u8, usize),
    Right(usize),
    Left(usize),
    Out(usize),
    In(usize),
    Loop(Vec<Expr>),
    Clear(usize),
    ScanRight,
    ScanLeft,
}

pub mod parser {
    include!(concat!(env!("OUT_DIR"), "/grammar.rs"));
}
