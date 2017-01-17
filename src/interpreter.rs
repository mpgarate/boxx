use parser::parser::{parse};
use expr::Expr;
use expr::Expr::*;
use expr::Val::*;
use expr::UnOp::*;
use expr::BinOp::*;
use expr::Dec::*;
use state::State;
use runtime_error::RuntimeError;
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
      Var(x) => {
        match self.state.get(x.clone()) {
          Some(e) => e,
          None => return Err(RuntimeError::VariableNotFound(x)),
        }
      },
      /**
       * Values are ineligible for step
       */
      Val(_) => {
        debug!("stepping on a value {:?}", e);
        return Err(RuntimeError::SteppingOnValue(e));
      },
      /**
       * Base cases
       */
      Uop(Not, box Val(Bool(b))) => {
        Val(Bool(!b))
      },
      Uop(Neg, box Val(Int(n))) => {
        Val(Int(-1 * n))
      },
      Bop(And, box Val(Bool(b1)), box Val(Bool(b2))) => {
        Val(Bool(b1 && b2))
      },
      Bop(Or, box Val(Bool(b1)), box Val(Bool(b2))) => {
        Val(Bool(b1 || b2))
      },
      Bop(Eq, ref e1, ref e2) if e1.is_value() && e2.is_value() => {
        Val(Bool(*e1 == *e2))
      },
      Bop(Ne, ref e1, ref e2) if e1.is_value() && e2.is_value() => {
        Val(Bool(*e1 != *e2))
      },
      Bop(Mod, box Val(Int(n1)), box Val(Int(n2))) => {
        // rust % gives the remainder, not modulus
        // let result = ((n1 % n2) + n2) % n2;
        let result = n1.checked_rem(n2)
          .ok_or(RuntimeError::IntegerUnderflow)?
          .checked_add(n2)
          .ok_or(RuntimeError::IntegerOverflow)?
          .checked_rem(n2)
          .ok_or(RuntimeError::IntegerUnderflow)?;

        Val(Int(result))
      },
      Bop(Lt, box Val(Int(n1)), box Val(Int(n2))) => {
        Val(Bool(n1 < n2))
      },
      Bop(Gt, box Val(Int(n1)), box Val(Int(n2))) => {
        Val(Bool(n1 > n2))
      },
      Bop(Leq, box Val(Int(n1)), box Val(Int(n2))) => {
        Val(Bool(n1 <= n2))
      },
      Bop(Geq, box Val(Int(n1)), box Val(Int(n2))) => {
        Val(Bool(n1 >= n2))
      },
      Bop(Plus, box Val(Int(n1)), box Val(Int(n2))) => {
        match n1.checked_add(n2) {
          Some(n) => Val(Int(n)),
          None => return Err(RuntimeError::IntegerOverflow),
        }
      },
      Bop(Minus, box Val(Int(n1)), box Val(Int(n2))) => {
        match n1.checked_sub(n2) {
          Some(n) => Val(Int(n)),
          None => return Err(RuntimeError::IntegerUnderflow),
        }
      },
      Bop(Times, box Val(Int(n1)), box Val(Int(n2))) => {
        match n1.checked_mul(n2) {
          Some(n) => Val(Int(n)),
          None => return Err(RuntimeError::IntegerOverflow),
        }
      },
      Bop(Div, box Val(Int(n1)), box Val(Int(n2))) => {
        match n1.checked_div(n2) {
          Some(n) => Val(Int(n)),
          None => return Err(RuntimeError::IntegerUnderflow),
        }
      },
      Bop(Seq, box Val(_), box e1) => {
        e1
      },
      Bop(Assign, box Var(x), box Val(v)) => {
        self.state.assign(x, Val(v.clone()))?;
        debug!("done assigning {:?}", self.state.mem);
        Val(v)
      },
      Ternary(box Val(Bool(b)), box e1, box e2) => {
        match b {
          true => e1,
          false => e2,
        }
      },
      Decl(DConst, box Var(x), box Val(v1), box e1) => {
        self.state.alloc_const(x, Val(v1))?;
        e1
      },
      Decl(DVar, box Var(x), box Val(v1), box e1) => {
        debug!("allocing {:?}", v1);
        self.state.alloc(x, Val(v1))?;
        e1
      },
      // lambda lift so we can use iter() in guard
      // https://github.com/rust-lang/rfcs/issues/1006
      FnCall(ref v1, ref es) if v1.is_func() && (|| es.iter().all(|v| v.is_value()))() => {
        match **v1 {
          Val(Func(ref name, ref e1, ref xs)) => {
            self.state.begin_scope();

            // alloc the params
            for (xn, en) in xs.iter().zip(es.iter()) {
              self.state.alloc(xn.to_var()?, en.clone())?;
            }

            // alloc the fn body for named functions
            match *name {
              Some(ref s) => self.state.alloc(s.to_var()?, *v1.clone())?,
              _ => {},
            };

            Scope(Box::new(*e1.clone()))
          },
          _ => return Err(RuntimeError::UnexpectedExpr("expected Func".to_string(), *v1.clone()))
        }
      },
      Scope(box Val(v)) => {
        self.state.end_scope();
        Val(v)
      },
      While(box Val(Bool(b)), e1o, _, e2o, e3) => {
        match b {
          true => While(e1o.clone(), e1o, e2o.clone(), e2o, e3),
          false => *e3,
        }
      },
      Print(box Val(v)) => {
        println!("{}", v);
        Val(Undefined)
      },
      /**
       * Search Cases
       */
      Bop(op, box Val(v1), box e2) => {
        Bop(
          op,
          Box::new(Val(v1)),
          Box::new(self.step(e2)?)
        )
      },
      Bop(Assign, box Var(v1), box e2) => {
        Bop(
          Assign,
          Box::new(Var(v1)),
          Box::new(self.step(e2)?)
        )
      },
      Bop(op, box e1, e2) => {
        Bop(op, Box::new(self.step(e1)?), e2)
      },
      Uop(op, box e1) => {
        Uop(op, Box::new(self.step(e1)?))
      },
      Ternary(box e1, e2, e3) => {
        Ternary(Box::new(self.step(e1)?), e2, e3)
      },
      While(box e1, e1o, box Val(v), e2o, e3) => {
        While(Box::new(self.step(e1)?), e1o, Box::new(Val(v)), e2o, e3)
      },
      While(e1, e1o, box e2, e2o, e3) => {
        While(e1, e1o, Box::new(self.step(e2)?), e2o, e3)
      },
      Decl(dt, box addr, box e1, e2) => {
        Decl(dt, Box::new(addr.clone()), Box::new(self.step(e1)?), e2)
      },
      FnCall(f @ box Val(Func(_, _, _)), args) => {
        let mut found_nonvalue = false;

        let stepped_args: Result<Vec<Expr>> = args.iter().map(|e| {
          if !found_nonvalue && !e.is_value() {
            found_nonvalue = true;
            self.step(e.clone())
          } else {
            Ok(e.clone())
          }
        }).collect();

        FnCall(f, stepped_args?)
      },
      FnCall(box e1, args) => {
        FnCall(Box::new(self.step(e1)?), args)
      },
      Scope(box e1) => {
        Scope(Box::new(self.step(e1)?))
      },
      Print(box e1) => {
        Print(Box::new(self.step(e1)?))
      },
    };

    debug!("returning with mem {:?}" , self.state.mem);
    debug!("returning with e {:?}" , e1);
    Ok(e1)
  }

  pub fn eval(&mut self, input: &str) -> Result<Expr> {
    let mut e = parse(input)?;

    let mut num_iterations = 0;

    loop {
      if num_iterations > 1000000000 {
        return Err(RuntimeError::TooManyIterations(num_iterations))
      }

      debug!("-----------------");
      debug!("--- iterating on e {:?} ", e);
      debug!("--- iterating on m {:?} ", self.state.mem);
      num_iterations += 1;
      if e.is_value() {
        debug!("--- iterations: {}", num_iterations);
        return Ok(e.clone());
      } else {
        e = self.step(e.clone())?;
      }
    }
  }
}
