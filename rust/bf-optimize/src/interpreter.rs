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
                Expr::Block(commands) => {
                    self.eval(&commands);
                }
                Expr::AddMul(offset, coeff) => {
                    let p = self.memory[self.address];
                    let addr = self.address + *offset;
                    let now = self.memory[addr];
                    self.memory[addr] = now.wrapping_add(p.wrapping_mul(*coeff));
                }
                Expr::SubMul(offset, coeff) => {
                    let p = self.memory[self.address];
                    let addr = self.address + *offset;
                    let now = self.memory[addr];
                    self.memory[addr] = now.wrapping_sub(p.wrapping_mul(*coeff));
                }
            }
        }
    }
}
