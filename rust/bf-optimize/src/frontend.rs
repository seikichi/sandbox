#[derive(Debug, PartialEq, Eq)]
pub enum Expr {
    Add(u8, usize, bool),
    Sub(u8, usize, bool),
    Right(usize),
    Left(usize),
    Out(usize, bool),
    In(usize, bool),
    Loop(Vec<Expr>),
    Clear(usize, bool),
    ScanRight,
    ScanLeft,
}

pub mod parser {
    include!(concat!(env!("OUT_DIR"), "/grammar.rs"));
}
