use super::frontend::*;

pub struct Optimizer {}

impl Optimizer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn optimize(&mut self, commands: &mut Vec<Expr>) {
        let mut start = 0;
        for i in 0..commands.len() {
            if let Expr::Loop(_) = commands[i] {
                self.optimize_lazy_move(&mut commands[start..i]);
                start = i + 1;
            }

            if let Expr::Loop(children) = &mut commands[i] {
                self.optimize(children);
                if self.can_convert_to_multiplication(children) {
                    commands[i] = self.optimize_multiplication(children);
                }
            }
        }
        self.optimize_lazy_move(&mut commands[start..]);
    }

    fn can_convert_to_multiplication(&mut self, commands: &[Expr]) -> bool {
        // commands contains Expr::Add only
        let only_add = commands.iter().all(|c| match c {
            Expr::Add(_, _) | Expr::Nop => true,
            _ => false,
        });
        if !only_add {
            return false;
        }

        // commands contains a Expr::Add(-1, 0)
        let mut zeros = commands.iter().filter(|c| match c {
            Expr::Add(_, 0) => true,
            _ => false,
        });
        if zeros.next() != Some(&Expr::Add(-1, 0)) {
            return false;
        }
        if zeros.next() != None {
            return false;
        }

        true
    }

    fn optimize_multiplication(&mut self, commands: &[Expr]) -> Expr {
        let mut new = vec![];
        for c in commands {
            match c {
                Expr::Add(count, offset) if *offset != 0 => {
                    new.push(Expr::Mul(*count, *offset));
                }
                _ => (),
            }
        }
        new.push(Expr::Clear(0));
        Expr::Block(new)
    }

    fn optimize_lazy_move(&mut self, commands: &mut [Expr]) {
        let mut current_offset = 0isize;
        let mut j = 0;

        for i in 0..commands.len() {
            match &commands[i] {
                Expr::Move(offset) => {
                    current_offset += offset;
                }
                Expr::Add(count, offset) => {
                    commands[j] = Expr::Add(*count, current_offset + offset);
                    j += 1;
                }
                Expr::Out(offset) => {
                    commands[j] = Expr::Out(current_offset + offset);
                    j += 1;
                }
                Expr::In(offset) => {
                    commands[j] = Expr::In(current_offset + offset);
                    j += 1;
                }
                Expr::Clear(offset) => {
                    commands[j] = Expr::Clear(current_offset + offset);
                    j += 1;
                }
                Expr::Nop => {}
                _ => panic!("optimize_lazy_move meets an unsupported command"),
            }
        }

        if current_offset != 0 {
            commands[j] = Expr::Move(current_offset);
            j += 1;
        }

        for c in &mut commands[j..] {
            *c = Expr::Nop;
        }
    }
}

#[test]
fn test_put_h_optimize() {
    let mut commands = vec![
        Expr::Move(1),
        Expr::Add(9, 0),
        Expr::Loop(vec![
            Expr::Move(-1),
            Expr::Add(8, 0),
            Expr::Move(1),
            Expr::Add(-1, 0),
        ]),
        Expr::Move(-1),
        Expr::Out(0),
    ];

    let mut optimizer = Optimizer::new();
    optimizer.optimize(&mut commands);

    let expected = vec![
        Expr::Add(9, 1),
        Expr::Move(1),
        Expr::Block(vec![Expr::Mul(8, -1), Expr::Clear(0)]),
        Expr::Out(-1),
        Expr::Move(-1),
    ];

    assert_eq!(commands, expected);
}
