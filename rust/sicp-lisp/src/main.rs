#![feature(box_syntax, box_patterns)]

mod environment;
mod eval;
mod sexp;
mod value;

use environment::*;
use eval::*;
use sexp::*;

use std::io::prelude::*;
use std::io::{self, BufReader};

fn main() {
    let env = Environment::new();

    let prompt = || {
        print!("> ");
        io::stdout().flush().unwrap();
    };
    prompt();

    let stdin = io::stdin();
    let reader = BufReader::new(stdin.lock());

    for line in reader.lines() {
        let e = parser::expression(&line.unwrap()).unwrap();
        println!("{:?}", eval(&e, &env));
        prompt();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_square() {
        let s1 = "(define square (lambda (x) (* x x)))";
        let s2 = "(square 10)";
        let e1 = parser::expression(s1).unwrap();
        let e2 = parser::expression(s2).unwrap();

        let env = Environment::new();
        assert_eq!(format!("{:?}", eval(&e1, &env)), "ok");
        assert_eq!(format!("{:?}", eval(&e2, &env)), "100");
    }

    #[test]
    fn test_factorial() {
        let s1 = "(define factorial (lambda (n) (if (= n 0) 1 (* n (factorial (- n 1))))))";
        let s2 = "(factorial 5)";
        let e1 = parser::expression(s1).unwrap();
        let e2 = parser::expression(s2).unwrap();

        let env = Environment::new();
        assert_eq!(format!("{:?}", eval(&e1, &env)), "ok");
        assert_eq!(format!("{:?}", eval(&e2, &env)), "120");
    }

    #[test]
    fn test_tarai() {
        let s1 = "(define tarai (lambda (x y z) (if (<= x y) y (tarai (tarai (- x 1) y z) (tarai (- y 1) z x) (tarai (- z 1) x y)))))";
        let s2 = "(tarai 6 3 0)";

        let e1 = parser::expression(s1).unwrap();
        let e2 = parser::expression(s2).unwrap();

        let env = Environment::new();
        assert_eq!(format!("{:?}", eval(&e1, &env)), "ok");
        assert_eq!(format!("{:?}", eval(&e2, &env)), "6");
    }

    #[test]
    fn test_counter() {
        let s1 = "(define make-counter (lambda () (begin (define count 0) (lambda () (begin (set! count (+ count 1)) count)))))";
        let s2 = "(define c1 (make-counter))";
        let s3 = "(define c2 (make-counter))";
        let s4 = "(c1)";
        let s5 = "(c1)";
        let s6 = "(c2)";

        let e1 = parser::expression(s1).unwrap();
        let e2 = parser::expression(s2).unwrap();
        let e3 = parser::expression(s3).unwrap();
        let e4 = parser::expression(s4).unwrap();
        let e5 = parser::expression(s5).unwrap();
        let e6 = parser::expression(s6).unwrap();

        let env = Environment::new();
        assert_eq!(format!("{:?}", eval(&e1, &env)), "ok");
        assert_eq!(format!("{:?}", eval(&e2, &env)), "ok");
        assert_eq!(format!("{:?}", eval(&e3, &env)), "ok");
        assert_eq!(format!("{:?}", eval(&e4, &env)), "1");
        assert_eq!(format!("{:?}", eval(&e5, &env)), "2");
        assert_eq!(format!("{:?}", eval(&e6, &env)), "1");
    }

    #[test]
    fn test_list() {
        let s1 = "(define s1 (cons 1 (cons 2 (cons 3 (cons 4 nil)))))";
        let s2 = "(car s1)";
        let s3 = "(cdr s1)";
        let s4 = "(define sum (lambda (s) (if (null? s) 0 (+ (car s) (sum (cdr s))))))";
        let s5 = "(sum s1)";

        let e1 = parser::expression(s1).unwrap();
        let e2 = parser::expression(s2).unwrap();
        let e3 = parser::expression(s3).unwrap();
        let e4 = parser::expression(s4).unwrap();
        let e5 = parser::expression(s5).unwrap();

        let env = Environment::new();
        assert_eq!(format!("{:?}", eval(&e1, &env)), "ok");
        assert_eq!(format!("{:?}", eval(&e2, &env)), "1");
        assert_eq!(format!("{:?}", eval(&e3, &env)), "(2 . (3 . (4 . nil)))");
        assert_eq!(format!("{:?}", eval(&e4, &env)), "ok");
        assert_eq!(format!("{:?}", eval(&e5, &env)), "10");
    }
}
