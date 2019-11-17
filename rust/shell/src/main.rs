use std::env::current_dir;
use std::error::Error;
use std::ffi::CString;
use std::fs::File;
use std::io::prelude::*;
use std::io::{stdin, stdout};
use std::os::unix::io::{AsRawFd, IntoRawFd};

use dirs::home_dir;
use nix::sys::wait::wait;
use nix::unistd::{chdir, close, dup2, execvp, fork, pipe, ForkResult};
use peg::parser;

fn main() -> Result<(), Box<dyn Error>> {
    let stdin = stdin();
    let mut lines = stdin.lock().lines();

    loop {
        print!("{} $ ", current_dir()?.to_str().unwrap());
        stdout().flush()?;

        let next = lines.next();
        if next.is_none() {
            break;
        }
        let line = next.unwrap()?;
        if line.trim().is_empty() {
            continue;
        }

        let command = shell::command(&line).unwrap();

        if let Command::Execute { argv } = &command {
            if argv[0] == "exit" {
                break;
            }
            if argv[0] == "cd" && argv.len() == 1 {
                let path = home_dir().unwrap();
                if chdir(&path).is_err() {
                    eprintln!("failed to change directory");
                }
                continue;
            }
            if argv[0] == "cd" && argv.len() == 2 {
                if chdir(argv[1].as_str()).is_err() {
                    eprintln!("failed to change directory");
                }
                continue;
            }
        }

        match fork() {
            Ok(ForkResult::Parent { .. }) => {
                wait()?;
            }
            Ok(ForkResult::Child) => {
                command.run()?;
            }
            Err(_) => panic!("fork failed"),
        }
    }

    Ok(())
}

#[derive(Debug, Eq, PartialEq)]
pub enum Command {
    Execute {
        argv: Vec<String>,
    },
    Pipe {
        left: Box<Command>,
        right: Box<Command>,
    },
    Redirect {
        command: Box<Command>,
        op: char,
        file: String,
    },
}

impl Command {
    pub fn run(&self) -> Result<(), Box<dyn Error>> {
        match &self {
            Command::Execute { argv } => {
                let path = CString::new(argv[0].clone())?;
                let argv = argv
                    .iter()
                    .map(|s| CString::new(s.as_str()).unwrap())
                    .collect::<Vec<CString>>();
                let result = execvp(&path, &argv);
                if result.is_err() {
                    eprintln!("Error: {:?}", result);
                }
            }
            Command::Redirect { op, file, command } => {
                if *op == '<' {
                    let file = File::open(file)?;
                    let fd = file.into_raw_fd();
                    dup2(fd, stdin().as_raw_fd())?;
                    command.run()?;
                } else if *op == '>' {
                    let file = File::create(file)?;
                    let fd = file.into_raw_fd();
                    dup2(fd, stdout().as_raw_fd())?;
                    command.run()?;
                } else {
                    command.run()?;
                }
            }
            Command::Pipe { left, right } => {
                let (input, output) = pipe()?;
                match fork() {
                    Ok(ForkResult::Parent { .. }) => {
                        close(output)?;
                        dup2(input, stdin().as_raw_fd())?;
                        right.run()?;
                    }
                    Ok(ForkResult::Child) => {
                        close(input)?;
                        dup2(output, stdout().as_raw_fd())?;
                        left.run()?;
                    }
                    Err(_) => panic!("fork failed"),
                }
            }
        }

        Ok(())
    }
}

fn parse_redirect(tokens: &[String]) -> Command {
    let mut argv = tokens.to_vec();

    if let Some(pos) = argv.iter().position(|t| t == "<" || t == ">") {
        let op = argv.remove(pos).chars().next().unwrap();
        let file = argv.remove(pos);
        let command = Box::new(parse_redirect(&argv));
        Command::Redirect { op, file, command }
    } else {
        Command::Execute { argv }
    }
}

parser! {
    grammar shell() for str {
        pub rule command() -> Command
          = pipe:pipe() { pipe }
          / exec:execute() { exec }

        rule execute() -> Command
          = argv:(sep() arg:token() { arg })+ { parse_redirect(&argv) }

        rule pipe() -> Command
          = left:execute() sep() "|" right:command() {
              Command::Pipe { left: Box::new(left), right: Box::new(right) }
          }

        rule token() -> String
          = "<" { "<".into() }
          / ">" { ">".into() }
          / ts:(unquoted() / double_quoted() / single_quoted())+ { ts.join("") }

        rule unquoted() -> String
          = t:$((!['<' | '>' | '|' | ' ' | '"' | '\''] [_])+) { t.into() }

        // NOTE: don't support escape (e.g., "\"")
        rule double_quoted() -> String
          = ['"'] s:$((!['"'] [_])+) ['"'] { s.into() }

        // NOTE: don't support escape (e.g., '\'')
        rule single_quoted() -> String
          = "'" s:$((!['\''] [_])+) "'" { s.into() }

        rule sep() = [' ' | '\t' | '\n']*
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute() {
        let command = shell::command("echo foo bar").unwrap();
        assert_eq!(
            command,
            Command::Execute {
                argv: vec!["echo".into(), "foo".into(), "bar".into()]
            }
        );
    }

    #[test]
    fn test_escape() {
        let command = shell::command(r###"echo "FOO" 'BAR' "FOO"'BAR'"###).unwrap();
        assert_eq!(
            command,
            Command::Execute {
                argv: vec!["echo".into(), "FOO".into(), "BAR".into(), "FOOBAR".into()]
            }
        );
    }

    #[test]
    fn test_pipe() {
        let command = shell::command("ls | grep foo").unwrap();
        assert_eq!(
            command,
            Command::Pipe {
                left: Box::new(Command::Execute {
                    argv: vec!["ls".into()]
                }),
                right: Box::new(Command::Execute {
                    argv: vec!["grep".into(), "foo".into()]
                })
            }
        );
    }

    #[test]
    fn test_redirect() {
        let command = shell::command("echo HELLO > output.txt").unwrap();
        assert_eq!(
            command,
            Command::Redirect {
                op: '>',
                file: "output.txt".into(),
                command: Box::new(Command::Execute {
                    argv: vec!["echo".into(), "HELLO".into()]
                }),
            }
        );
    }

    #[test]
    fn test_complex_redirect() {
        let command = shell::command("echo A B>X C").unwrap();
        assert_eq!(
            command,
            Command::Redirect {
                op: '>',
                file: "X".into(),
                command: Box::new(Command::Execute {
                    argv: vec!["echo".into(), "A".into(), "B".into(), "C".into()]
                }),
            }
        );
    }

    #[test]
    fn test_complex() {
        let command =
            shell::command("cat < input.txt | grep keyword | grep -v exclude > output.txt")
                .unwrap();
        assert_eq!(
            command,
            Command::Pipe {
                left: Box::new(Command::Redirect {
                    op: '<',
                    file: "input.txt".into(),
                    command: Box::new(Command::Execute {
                        argv: vec!["cat".into()]
                    })
                }),
                right: Box::new(Command::Pipe {
                    left: Box::new(Command::Execute {
                        argv: vec!["grep".into(), "keyword".into()]
                    }),
                    right: Box::new(Command::Redirect {
                        op: '>',
                        file: "output.txt".into(),
                        command: Box::new(Command::Execute {
                            argv: vec!["grep".into(), "-v".into(), "exclude".into()]
                        }),
                    })
                })
            }
        );
    }
}
