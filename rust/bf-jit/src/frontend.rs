#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Expr {
    // Add(count, offset): data[ptr + offset] += count
    Add(i32, isize),

    // Move(offset): ptr += offset
    Move(isize),

    // Out(offset): putchar(ptr + offset)
    Out(isize),

    // Out(offset): getchar(ptr + offset)
    In(isize),

    // Loop(commands): while (data[ptr] != 0) { commands }
    Loop(Vec<Expr>),

    // Clear(offset): data[ptr + offset] = 0
    Clear(isize),

    // ==================================================================
    // The following operations will be generated by optimizer::Optimizer
    // ==================================================================

    // Mul(count, index, offset): data[ptr + index + offset] = count * data[ptr + offset]
    Mul(i32, isize, isize),

    // Nop:
    Nop,
}

pub mod parser {
    include!(concat!(env!("OUT_DIR"), "/grammar.rs"));
}
