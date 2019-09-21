#![feature(box_syntax, box_patterns)]

mod environment;
mod sexp;
mod value;

use std::rc::Rc;

use environment::*;
use sexp::*;
use value::*;

fn eval(exp: &Sexp, env: &Rc<Environment>) -> Rc<Value> {
    match exp {
        Sexp::Integer(n) => Rc::new(Value::Integer(*n)),
        Sexp::Symbol(s) => env.lookup(s),
        Sexp::Pair(
            box Sexp::Symbol(s),
            box Sexp::Pair(box Sexp::Symbol(var), box Sexp::Pair(box val, box Sexp::Nil)),
        ) if s == "set!" => {
            env.set(&var, eval(val, env));
            Rc::new(Value::Symbol("ok".to_string()))
        }
        Sexp::Pair(
            box Sexp::Symbol(s),
            box Sexp::Pair(box Sexp::Symbol(var), box Sexp::Pair(box val, box Sexp::Nil)),
        ) if s == "define" => {
            env.define(&var, eval(val, env));
            Rc::new(Value::Symbol("ok".to_string()))
        }
        Sexp::Pair(box Sexp::Symbol(s), box actions) if s == "begin" => eval_sequence(actions, env),
        Sexp::Pair(box Sexp::Symbol(s), box Sexp::Pair(p, b)) if s == "lambda" => {
            let mut parameters = vec![];

            let mut next = &**p;
            while let Sexp::Pair(car, cdr) = next {
                if let Sexp::Symbol(s) = &**car {
                    parameters.push(s.clone());
                }
                next = cdr;
            }

            let mut body = vec![];
            let mut next = &**b;
            while let Sexp::Pair(car, cdr) = next {
                body.push(*car.clone());
                next = cdr;
            }

            Rc::new(Value::CompoundProcedure {
                body,
                parameters,
                environment: env.clone(),
            })
        }
        Sexp::Pair(operator, operands) => {
            apply(eval(operator, env), &list_of_values(operands, env))
        }
        _ => panic!("Unknown expression type -- EVAL {:?}", exp),
    }
}

fn eval_sequence(exp: &Sexp, env: &Rc<Environment>) -> Rc<Value> {
    let mut result = Rc::new(Value::Nil);
    let mut p = exp;
    while let Sexp::Pair(car, cdr) = p {
        result = eval(car, env);
        p = cdr;
    }
    result
}

fn list_of_values(exps: &Sexp, env: &Rc<Environment>) -> Vec<Rc<Value>> {
    let mut values = vec![];
    let mut p = exps;
    while let Sexp::Pair(car, cdr) = p {
        values.push(eval(car, env));
        p = cdr;
    }
    values
}

fn apply(procedure: Rc<Value>, arguments: &[Rc<Value>]) -> Rc<Value> {
    match &*procedure {
        Value::PrimitiveProcedure(f) => Rc::new(f(arguments)),
        Value::CompoundProcedure {
            parameters,
            body,
            environment,
        } => {
            let env = Environment::extend(&parameters, arguments, &environment);
            let mut result = Rc::new(Value::Nil);
            for b in body {
                result = eval(b, &env);
            }
            result
        }
        _ => panic!("Invalid procedure: {:?}", procedure),
    }
}

fn main() {
    let env = Environment::empty();
    env.define(
        "+",
        Rc::new(Value::PrimitiveProcedure(Box::new(|args| {
            let mut result = 0;
            for arg in args {
                if let Value::Integer(n) = &**arg {
                    result += *n;
                } else {
                    panic!("Invalid arguments for +: {:?}", arg);
                }
            }
            Value::Integer(result)
        }))),
    );

    env.define(
        "display",
        Rc::new(Value::PrimitiveProcedure(Box::new(|args| {
            if args.len() != 1 {
                panic!("Invalid arguments for display: {:?}", args);
            }
            println!("{:?}", args[0]);
            Value::Nil
        }))),
    );

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
";

    for e in &parser::expressions(s).unwrap() {
        eval(e, &env);
    }
}
