use parser::parser::{parse};
use ast::Expr::*;
use ast::UnOp::*;
use ast::BinOp::*;
use ast::Dec::*;
use ast::*;
use state::*;
use interpreter_error::InterpreterError;
use std::result;

pub type Result<T> = result::Result<T, InterpreterError>;

macro_rules! step {
  ($s:expr, $e:expr) => (try!($s.step($e)));
}

pub struct Repl {
  pub state: State,
}

impl Repl {
  pub fn new() -> Repl {
    Repl {
      state: State::new(),
    }
  }

  pub fn step(&mut self, e: Expr) -> Result<Expr> {
    debug!("step(e) : {:?}", e);
    debug!("step(self.state) : {:?}", self.state.mem);

    let e1 = match e.clone() {
      Var(x) => {
        self.state.get(x)
      },
      /**
       * Values are ineligible for step
       */
      Int(_) | Bool(_) | Func(_, _, _) | Undefined => {
        debug!("stepping on a value {:?}", e);
        return Err(InterpreterError::SteppingOnValue(e));
      },
      /**
       * Base cases
       */
      Uop(Not, ref e1) if e1.is_bool() => {
        Bool(!e1.to_bool())
      },
      Uop(Neg, ref e1) if e1.is_int() => {
        Int(-1 * e1.to_int())
      },
      Bop(And, ref e1, ref e2) if e1.is_bool() && e2.is_bool() => {
        Bool(e1.to_bool() && e2.to_bool())
      },
      Bop(Or, ref e1, ref e2) if e1.is_bool() && e2.is_bool() => {
        Bool(e1.to_bool() || e2.to_bool())
      },
      Bop(Eq, ref e1, ref e2) if e1.is_value() && e2.is_value() => {
        Bool(*e1 == *e2)
      },
      Bop(Ne, ref e1, ref e2) if e1.is_value() && e2.is_value() => {
        Bool(*e1 != *e2)
      },
      Bop(Mod, ref e1, ref e2) if e1.is_int() && e2.is_int() => {
        let n1 = e1.to_int();
        let n2 = e2.to_int();

        // rust % gives the remainder, not modulus
        let result = ((n1 % n2) + n2) % n2;

        Int(result)
      },
      Bop(Lt, ref e1, ref e2) if e1.is_int() && e2.is_int() => {
        Bool(e1.to_int() < e2.to_int())
      },
      Bop(Gt, ref e1, ref e2) if e1.is_int() && e2.is_int() => {
        Bool(e1.to_int() > e2.to_int())
      },
      Bop(Leq, ref e1, ref e2) if e1.is_int() && e2.is_int() => {
        Bool(e1.to_int() <= e2.to_int())
      },
      Bop(Geq, ref e1, ref e2) if e1.is_int() && e2.is_int() => {
        Bool(e1.to_int() >= e2.to_int())
      },
      Bop(Plus, ref e1, ref e2) if e1.is_int() && e2.is_int() => {
        Int(e1.to_int() + e2.to_int())
      },
      Bop(Minus, ref e1, ref e2) if e1.is_int() && e2.is_int() => {
        Int(e1.to_int() - e2.to_int())
      },
      Bop(Times, ref e1, ref e2) if e1.is_int() && e2.is_int() => {
        Int(e1.to_int() * e2.to_int())
      },
      Bop(Div, ref e1, ref e2) if e1.is_int() && e2.is_int() => {
        Int(e1.to_int() / e2.to_int())
      },
      Bop(Seq, ref v1, ref e2) if v1.is_value() => {
        *e2.clone()
      },
      Bop(Assign, ref v1, ref v2) if v1.is_var() && v2.is_value() => {
        let x = v1.to_var();
        self.state.assign(x, *v2.clone());
        debug!("done assigning {:?}", self.state.mem);
        *v2.clone()
      },
      Ternary(ref v1, ref e2, ref e3) if v1.is_value() => {
        match v1.to_bool() {
          true => *e2.clone(),
          false => *e3.clone(),
        }
      },
      Decl(DConst, ref x, ref v1, ref e2) if v1.is_value() => {
        self.state.alloc_const(x.to_var(), *v1.clone());
        *e2.clone()
      },
      Decl(DVar, ref x, ref v1, ref e2) if x.is_var() && v1.is_value() => {
        debug!("allocing {:?}", v1);
        self.state.alloc(x.to_var(), *v1.clone());
        *e2.clone()
      },
      // lambda lift so we can use iter() in guard
      // https://github.com/rust-lang/rfcs/issues/1006
      FnCall(ref v1, ref es) if v1.is_func() && (|| es.iter().all(|v| v.is_value()))() => {
        match **v1 {
          Func(ref name, ref e1, ref xs) => {
            self.state.begin_scope();
            // sub the params
            let exp = xs.iter().zip(es.iter())
              .fold(*e1.clone(), |exp, (xn, en)| {
                self.state.alloc(xn.to_var(), en.clone());
                exp
              });

            // sub the fn body for named functions
            let body = match *name {
              None => exp,
              Some(ref s) => {
                self.state.alloc(s.to_var(), *v1.clone());
                exp
              }
            };
            Scope(Box::new(body))
          },
          _ => {
            debug!("expected a Func, got {:?}", v1);
            // TODO: return an error value here
            panic!()
          },
        }
      },
      Scope(ref v1) if v1.is_value() => {
        self.state.end_scope();
        *v1.clone()
      },
      /**
       * Search Cases
       */
      Bop(ref op, ref v1, ref e2) if v1.is_value() => {
        Bop(
          op.clone(),
          Box::new(*v1.clone()),
          Box::new(step!(self, *e2.clone()))
        )
      },
      Bop(Assign, ref v1, ref e2) if v1.is_var() => {
        Bop(
          Assign,
          Box::new(*v1.clone()),
          Box::new(step!(self, *e2.clone()))
        )
      },
      Bop(op, e1, e2) => {
        Bop(op, Box::new(step!(self, *e1)), e2)
      },
      Uop(op, e1) => {
        Uop(op, Box::new(step!(self, *e1)))
      },
      Ternary(e1, e2, e3) => {
        Ternary(Box::new(step!(self, *e1)), e2, e3)
      },
      While(ref v1, ref e1o, _, ref e2o, ref e3) if v1.is_value() => {
        match v1.to_bool() {
          true => While(Box::new(*e1o.clone()), e1o.clone(), e2o.clone(), e2o.clone(), e3.clone()),
          false => *e3.clone(),
        }
      },
      While(ref e1, ref e1o, ref v2, ref e2o, ref e3) if v2.is_value() => {
        While(Box::new(step!(self, *e1.clone())), e1o.clone(), v2.clone(), e2o.clone(), e3.clone())
      },
      While(e1, e1o, e2, e2o, e3) => {
        While(e1, e1o, Box::new(step!(self, *e2)), e2o, e3)
      },
      Decl(dt, addr, e1, e2) => {
        Decl(dt, Box::new(*addr.clone()), Box::new(step!(self, *e1)), e2)
      },
      FnCall(ref v1, ref args) if v1.is_value() => {
        let mut found_nonvalue = false;

        let args2 = args.iter().map(|e| {
          if !found_nonvalue && !e.is_value() {
            found_nonvalue = true;
            // TODO: handle error here
            self.step(e.clone()).unwrap()
          } else {
            e.clone()
          }
        }).collect();

        FnCall(v1.clone(), args2)
      },
      FnCall(e1, args) => {
        FnCall(Box::new(step!(self, *e1)), args)
      },
      Scope(e1) => {
        Scope(Box::new(step!(self, *e1)))
      },
    };

    debug!("returning with mem {:?}" , self.state.mem);
    debug!("returning with e {:?}" , e1);
    Ok(e1)
  }

  pub fn eval(&mut self, input: &str) -> Expr {
    let mut e = parse(input).expect("parser error");

    let mut num_iterations = 0;

    loop {
      if num_iterations > 1000 {
        // TODO: return an error value here
        panic!("too many step iterations");
      }

      debug!("-----------------");
      debug!("--- iterating on e {:?} ", e);
      debug!("--- iterating on m {:?} ", self.state.mem);
      num_iterations += 1;
      if e.is_value() {
        debug!("--- iterations: {}", num_iterations);
        return e.clone();
      } else {
        // TODO: consider handling error here
        e = self.step(e).unwrap();
      }
    }
  }
}

pub fn boxx(input: &str) -> Expr {
  let mut repl = Repl::new();
  repl.eval(input)
}
