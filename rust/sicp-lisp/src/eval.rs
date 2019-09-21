use std::rc::Rc;

use super::environment::*;
use super::sexp::*;
use super::value::*;

pub fn eval(exp: &Sexp, env: &Rc<Environment>) -> Rc<Value> {
    match exp {
        Sexp::Nil => Rc::new(Value::Nil),
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
        Sexp::Pair(
            box Sexp::Symbol(s),
            box Sexp::Pair(box p, box Sexp::Pair(box c, box Sexp::Pair(box a, box Sexp::Nil))),
        ) if s == "if" => eval(if is_true(&eval(p, env)) { c } else { a }, env),
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
    }
}

fn is_true(exp: &Rc<Value>) -> bool {
    !is_false(exp)
}

fn is_false(exp: &Rc<Value>) -> bool {
    if let Value::Symbol(s) = &**exp {
        s == "false"
    } else {
        false
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
        Value::PrimitiveProcedure(f) => f(arguments),
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
