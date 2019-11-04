use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum Json {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Object(Box<HashMap<String, Json>>),
}

pub mod parser {
    include!(concat!(env!("OUT_DIR"), "/grammar.rs"));
}
