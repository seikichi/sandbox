mod frontend;
mod jit;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::mem;

use jit::*;

fn main() -> std::io::Result<()> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        panic!("Usage: bf_jit [filename.bf]");
    }

    let source = {
        let mut text = String::new();
        let mut file = File::open(&args[1])?;
        file.read_to_string(&mut text)?;
        text.retain(|u| match u {
            '>' | '<' | '+' | '-' | '.' | ',' | '[' | ']' => true,
            _ => false,
        });
        text
    };

    let mut jit = JIT::new();
    let main = jit.compile(&source).unwrap();
    let main = unsafe { mem::transmute::<_, fn() -> ()>(main) };
    main();

    Ok(())
}
