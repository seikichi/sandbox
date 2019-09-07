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
                Expr::Right(count) => self.address += *count,
                Expr::Left(count) => self.address -= *count,
                Expr::Add(count, offset) => {
                    let addr = self.address + offset;
                    self.memory[addr] = self.memory[addr].wrapping_add(*count)
                }
                Expr::Sub(count, offset) => {
                    let addr = self.address + offset;
                    self.memory[addr] = self.memory[addr].wrapping_sub(*count)
                }
                Expr::Out(offset) => {
                    let addr = self.address + offset;
                    self.writer.write_all(&[self.memory[addr]]).unwrap();
                }
                Expr::In(offset) => {
                    let mut buf = vec![0; 1];
                    let _ = self.reader.read(&mut buf).unwrap();
                    self.memory[self.address + offset] = buf[0];
                }
                Expr::Loop(commands) => {
                    while self.memory[self.address] != 0 {
                        self.eval(&commands);
                    }
                }
                Expr::Clear(offset) => self.memory[self.address + offset] = 0,
                Expr::ScanRight => {
                    while self.memory[self.address] != 0 {
                        self.address += 1;
                    }
                }
                Expr::ScanLeft => {
                    while self.memory[self.address] != 0 {
                        self.address -= 1;
                    }
                }
            }
        }
    }
}
