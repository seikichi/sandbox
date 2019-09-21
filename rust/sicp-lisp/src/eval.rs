use std::rc::Rc;

use super::environment::*;
use super::sexp::*;
use super::value::*;

use super::sexp::Sexp::*;

pub fn eval(exp: &Sexp, env: &Rc<Environment>) -> Rc<Value> {
    match exp {
        Nil => Rc::new(Value::Nil),
        Integer(n) => Rc::new(Value::Integer(*n)),
        Symbol(s) => env.lookup(s),
        Pair(box Symbol(tag), box Pair(box Symbol(var), box Pair(box val, box Nil)))
            if tag == "set!" =>
        {
            env.set(&var, eval(val, env))
        }
        Pair(box Symbol(tag), box Pair(box Symbol(var), box Pair(box val, box Nil)))
            if tag == "define" =>
        {
            env.define(&var, eval(val, env))
        }
        Pair(box Symbol(tag), box Pair(box p, box Pair(box c, box Pair(box a, box Nil))))
            if tag == "if" =>
        {
            eval(if is_true(&eval(p, env)) { c } else { a }, env)
        }
        Pair(box Symbol(tag), box actions) if tag == "begin" => eval_sequence(actions, env),
        Pair(box Symbol(tag), box Pair(box p, box b)) if tag == "lambda" => eval_lambda(p, b, env),
        Pair(operator, operands) => apply(eval(operator, env), &list_of_values(operands, env)),
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

fn eval_lambda(p: &Sexp, b: &Sexp, env: &Rc<Environment>) -> Rc<Value> {
    let mut parameters = vec![];

    let mut next = &*p;
    while let Pair(car, cdr) = next {
        if let Symbol(s) = &**car {
            parameters.push(s.clone());
        }
        next = cdr;
    }

    let mut body = vec![];
    let mut next = &*b;
    while let Pair(car, cdr) = next {
        body.push(*car.clone());
        next = cdr;
    }

    Rc::new(Value::CompoundProcedure {
        body,
        parameters,
        environment: env.clone(),
    })
}

fn eval_sequence(exp: &Sexp, env: &Rc<Environment>) -> Rc<Value> {
    let mut result = Rc::new(Value::Nil);
    let mut p = exp;
    while let Pair(car, cdr) = p {
        result = eval(car, env);
        p = cdr;
    }
    result
}

fn list_of_values(exps: &Sexp, env: &Rc<Environment>) -> Vec<Rc<Value>> {
    let mut values = vec![];
    let mut p = exps;
    while let Pair(car, cdr) = p {
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
