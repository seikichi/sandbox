mod frontend;
mod interpreter;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{stdin, stdout, BufReader, BufWriter};

use frontend::*;
use interpreter::*;

fn main() -> std::io::Result<()> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        panic!("Usage: bf-simple [filename.bf]");
    }

    let stdout = stdout();
    let writer = BufWriter::new(stdout.lock());

    let stdin = stdin();
    let reader = BufReader::new(stdin.lock());

    let mut interpreter = Interpreter::new(reader, writer);

    let commands = {
        let mut text = String::new();
        let mut file = File::open(&args[1])?;
        file.read_to_string(&mut text)?;
        text.retain(|u| match u {
            '>' | '<' | '+' | '-' | '.' | ',' | '[' | ']' => true,
            _ => false,
        });
        parser::program(&text).unwrap()
    };

    interpreter.eval(&commands);

    Ok(())
}
