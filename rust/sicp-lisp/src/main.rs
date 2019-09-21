#![feature(box_syntax, box_patterns)]

mod environment;
mod eval;
mod sexp;
mod value;

use environment::*;
use eval::*;
use sexp::*;

fn main() {
    let env = Environment::new();

    let s = "
(define double (lambda (x) (+ x x)))
(display (double 21))

(define make-counter (lambda () (begin (define count 0) (lambda () (begin (set! count (+ count 1)) count)))))
(define c1 (make-counter))
(define c2 (make-counter))
(display (c1))
(display (c1))
(display (c1))
(display (c2))
(display (c2))

(define s1 (cons 1 (cons 2 nil)))
(display s1)
(display (car s1))
(display (cdr s1))
";

    for e in &parser::expressions(s).unwrap() {
        eval(e, &env);
    }
}
