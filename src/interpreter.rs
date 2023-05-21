use expr::BinOp::*;
use expr::Dec::*;
use expr::Expr;
use expr::Expr::*;
use expr::UnOp::*;
use expr::Val::*;
use parser::parser::parse;
use runtime_error::RuntimeError;
use state::State;
use std::mem;
use std::result;

pub type Result<T> = result::Result<T, RuntimeError>;

pub struct Interpreter {
    pub state: State,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            state: State::new(),
        }
    }

    pub fn step(&mut self, e: Expr) -> Result<Expr> {
        debug!("step(e) : {:?}", e);
        debug!("step(self.state) : {:?}", self.state.mem);

        let e1 = match e {
            Var(x) => match self.state.get(x.clone()) {
                Some(e) => e,
                None => return Err(RuntimeError::VariableNotFound(x)),
            },
            // Values are ineligible for step
            Val(_) => {
                debug!("stepping on a value {:?}", e);
                return Err(RuntimeError::SteppingOnValue(e));
            }
            Uop(op, e1) => match (op, *e1) {
                (Not, Val(Bool(b))) => Val(Bool(!b)),
                (Neg, Val(Int(n))) => Val(Int(-1 * n)),
                (op, e1) => Uop(op, Box::new(self.step(e1)?)),
            },
            Bop(op, e1, e2) => match (op, *e1, *e2) {
                (And, Val(Bool(b1)), Val(Bool(b2))) => Val(Bool(b1 && b2)),
                (Or, Val(Bool(b1)), Val(Bool(b2))) => Val(Bool(b1 || b2)),
                (Eq, Val(v1), Val(v2)) => Val(Bool(v1 == v2)),
                (Ne, Val(v1), Val(v2)) => Val(Bool(v1 != v2)),
                (Mod, Val(Int(n1)), Val(Int(n2))) => {
                    // rust % gives the remainder, not modulus
                    // let result = ((n1 % n2) + n2) % n2;
                    let result = n1
                        .checked_rem(n2)
                        .ok_or(RuntimeError::IntegerUnderflow)?
                        .checked_add(n2)
                        .ok_or(RuntimeError::IntegerOverflow)?
                        .checked_rem(n2)
                        .ok_or(RuntimeError::IntegerUnderflow)?;

                    Val(Int(result))
                }
                (Lt, Val(Int(n1)), Val(Int(n2))) => Val(Bool(n1 < n2)),
                (Gt, Val(Int(n1)), Val(Int(n2))) => Val(Bool(n1 > n2)),
                (Leq, Val(Int(n1)), Val(Int(n2))) => Val(Bool(n1 <= n2)),
                (Geq, Val(Int(n1)), Val(Int(n2))) => Val(Bool(n1 >= n2)),
                (Plus, Val(Int(n1)), Val(Int(n2))) => match n1.checked_add(n2) {
                    Some(n) => Val(Int(n)),
                    None => return Err(RuntimeError::IntegerOverflow),
                },
                (Minus, Val(Int(n1)), Val(Int(n2))) => match n1.checked_sub(n2) {
                    Some(n) => Val(Int(n)),
                    None => return Err(RuntimeError::IntegerUnderflow),
                },
                (Times, Val(Int(n1)), Val(Int(n2))) => match n1.checked_mul(n2) {
                    Some(n) => Val(Int(n)),
                    None => return Err(RuntimeError::IntegerOverflow),
                },
                (Div, Val(Int(n1)), Val(Int(n2))) => match n1.checked_div(n2) {
                    Some(n) => Val(Int(n)),
                    None => return Err(RuntimeError::IntegerUnderflow),
                },
                (Seq, Val(_), e1) => e1,
                (Assign, Var(x), v @ Val(_)) => {
                    self.state.assign(x, v.clone())?;
                    debug!("done assigning {:?}", self.state.mem);
                    v
                }
                // Search Cases
                (op, v1 @ Val(_), e2) => Bop(op, Box::new(v1), Box::new(self.step(e2)?)),
                (Assign, v1 @ Var(_), e2) => Bop(Assign, Box::new(v1), Box::new(self.step(e2)?)),
                (op, e1, e2) => Bop(op, Box::new(self.step(e1)?), Box::new(e2)),
            },
            Decl(dt, addr, e1, e2) => match (dt, *addr, *e1, *e2) {
                (DConst, Var(x), v1 @ Val(_), e1) => {
                    self.state.alloc_const(x, v1)?;
                    e1
                }
                (DVar, Var(x), v1 @ Val(_), e1) => {
                    debug!("allocing {:?}", v1);
                    self.state.alloc(x, v1)?;
                    e1
                }
                (dt, addr, e1, e2) => {
                    Decl(dt, Box::new(addr), Box::new(self.step(e1)?), Box::new(e2))
                }
            },
            While(e1, e1o, e2, e2o, e3) => match (*e1, e1o, *e2, e2o, e3) {
                (Val(Bool(b)), e1o, _, e2o, e3) => match b {
                    true => While(e1o.clone(), e1o, e2o.clone(), e2o, e3),
                    false => *e3,
                },
                (e1, e1o, v @ Val(_), e2o, e3) => {
                    While(Box::new(self.step(e1)?), e1o, Box::new(v), e2o, e3)
                }
                (e1, e1o, e2, e2o, e3) => {
                    While(Box::new(e1), e1o, Box::new(self.step(e2)?), e2o, e3)
                }
            },
            Print(e1) => match *e1 {
                Val(v) => {
                    println!("{}", v);
                    Val(Undefined)
                }
                e1 => Print(Box::new(self.step(e1)?)),
            },
            Ternary(e1, e2, e3) => match (*e1, e2, e3) {
                (Val(Bool(b)), e1, e2) => match b {
                    true => *e1,
                    false => *e2,
                },
                (e1, e2, e3) => Ternary(Box::new(self.step(e1)?), e2, e3),
            },
            FnCall(e1, args) => {
                match (*e1, args) {
                    (Val(Func(ref name, ref e1, ref xs)), ref es)
                        if (|| {
                            es.iter()
                                .all(|v| if let Val(_) = *v { true } else { false })
                        })() =>
                    {
                        self.state.begin_scope();

                        // alloc the params
                        for (xn, en) in xs.iter().zip(es.iter()) {
                            if let &Var(ref x) = xn {
                                self.state.alloc(x.to_string(), en.clone())?;
                            } else {
                                return Err(RuntimeError::InvalidTypeConversion(
                                    "var".to_string(),
                                    xn.clone(),
                                ));
                            }
                        }

                        // alloc the fn body for named functions
                        if let Some(n) = name {
                            if let Var(ref s) = **n {
                                self.state.alloc(
                                    s.clone(),
                                    Val(Func(name.clone(), e1.clone(), xs.clone())),
                                )?;
                            }
                        };

                        Scope(e1.clone())
                    }
                    (f @ Val(Func(_, _, _)), mut args) => {
                        // find the first nonvalue arg and call step() on it
                        if let Some(index) = args.iter().position(|e| match e {
                            &Val(_) => false,
                            _ => true,
                        }) {
                            // temporary placeholder so we can safely move the value
                            // this ensures good vec state in between when data is read and rewritten
                            let expr = mem::replace(&mut args[index], Val(Undefined));
                            args[index] = self.step(expr)?;
                        }

                        FnCall(Box::new(f), args)
                    }
                    (e1, args) => FnCall(Box::new(self.step(e1)?), args),
                    // lambda lift so we can use iter() in guard
                    // https://github.com/rust-lang/rfcs/issues/1006
                }
            }
            Scope(e1) => match *e1 {
                v @ Val(_) => {
                    self.state.end_scope();
                    v
                }
                e1 => Scope(Box::new(self.step(e1)?)),
            },
        };

        debug!("returning with mem {:?}", self.state.mem);
        debug!("returning with e {:?}", e1);
        Ok(e1)
    }

    pub fn eval(&mut self, input: &str) -> Result<Expr> {
        let mut e = parse(input)?;

        let mut num_iterations = 0;

        loop {
            if num_iterations > 1000000000 {
                return Err(RuntimeError::TooManyIterations(num_iterations));
            }

            debug!("-----------------");
            debug!("--- iterating on e {:?} ", e);
            debug!("--- iterating on m {:?} ", self.state.mem);
            num_iterations += 1;
            if let Val(_) = e {
                debug!("--- iterations: {}", num_iterations);
                return Ok(e);
            } else {
                e = self.step(e)?;
            }
        }
    }
}
