use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{stdin, stdout, BufReader, BufWriter};

fn main() -> std::io::Result<()> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        panic!("Usage: bf-simple [filename.bf]");
    }

    let stdout = stdout();
    let mut writer = BufWriter::new(stdout.lock());

    let stdin = stdin();
    let mut reader = BufReader::new(stdin.lock());

    let code = {
        let mut code = vec![];
        let mut file = File::open(&args[1])?;
        file.read_to_end(&mut code)?;
        code.retain(|u| match u {
            b'>' | b'<' | b'+' | b'-' | b'.' | b',' | b'[' | b']' => true,
            _ => false,
        });
        code
    };
    let mut counter = 0;
    let mut memory = [0u8; 65535];
    let mut address = 0;

    while counter < code.len() {
        match code[counter] {
            b'>' => address += 1,
            b'<' => address -= 1,
            b'+' => memory[address] = memory[address].wrapping_add(1),
            b'-' => memory[address] = memory[address].wrapping_sub(1),
            b'.' => writer.write_all(&[memory[address]])?,
            b',' => {
                let mut buf = vec![0; 1];
                let _ = reader.read(&mut buf)?;
                memory[address] = buf[0];
            }
            b'[' if memory[address] == 0 => {
                counter += 1;
                let mut loops = 0;
                while loops > 0 || code[counter] != b']' {
                    loops += match code[counter] {
                        b'[' => 1,
                        b']' => -1,
                        _ => 0,
                    };
                    counter += 1;
                }
            }
            b'[' => (),
            b']' => {
                counter -= 1;
                let mut loops = 0;
                while loops > 0 || code[counter] != b'[' {
                    loops += match code[counter] {
                        b']' => 1,
                        b'[' => -1,
                        _ => 0,
                    };
                    counter -= 1;
                }
                counter -= 1;
            }
            _ => panic!("invalid operation: {}", code[counter]),
        }
        counter += 1;
    }

    Ok(())
}
