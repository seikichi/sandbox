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
            }
        }
        self.optimize_lazy_move(&mut commands[start..]);
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
                Expr::Loop(_) => panic!("optimize_lazy_move meets Expr::Loop"),
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
        Expr::Loop(vec![
            Expr::Add(8, -1),
            Expr::Add(-1, 0),
            Expr::Nop,
            Expr::Nop,
        ]),
        Expr::Out(-1),
        Expr::Move(-1),
    ];
    assert_eq!(commands, expected);
}
