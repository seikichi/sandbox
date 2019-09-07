use super::Expr;
use std::io::prelude::*;

const MEMORY_SIZE: usize = 65535;

pub struct Interpreter<R, W> {
    address: usize,
    memory: [u8; MEMORY_SIZE],
    reader: R,
    writer: W,
}

impl<R: Read, W: Write> Interpreter<R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        Self {
            address: 0,
            memory: [0; MEMORY_SIZE],
            reader,
            writer,
        }
    }

    pub fn eval(&mut self, commands: &[Expr]) {
        for e in commands {
            match e {
                Expr::Right(count) => self.address += *count as usize,
                Expr::Left(count) => self.address -= *count as usize,
                Expr::Add(count) => {
                    self.memory[self.address] = self.memory[self.address].wrapping_add(*count)
                }
                Expr::Sub(count) => {
                    self.memory[self.address] = self.memory[self.address].wrapping_sub(*count)
                }
                Expr::Out => self.writer.write_all(&[self.memory[self.address]]).unwrap(),
                Expr::In => {
                    let mut buf = vec![0; 1];
                    let _ = self.reader.read(&mut buf).unwrap();
                    self.memory[self.address] = buf[0];
                }
                Expr::Loop(commands) => {
                    while self.memory[self.address] != 0 {
                        self.eval(&commands);
                    }
                }
                Expr::Clear => self.memory[self.address] = 0,
            }
        }
    }
}
