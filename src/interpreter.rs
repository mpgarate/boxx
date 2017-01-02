use parser::parser::{parse};
use expr::Expr;
use expr::Expr::*;
use expr::UnOp::*;
use expr::BinOp::*;
use expr::Dec::*;
use state::State;
use runtime_error::RuntimeError;
use std::result;
use std::mem::replace;

pub type Result<T> = result::Result<T, RuntimeError>;

macro_rules! get_and_replace_mem {
  ( $x:expr ) => {
    replace($x, Box::new(Undefined)) // arbitrary, just for the return value
  }
}

pub struct Interpreter {
  pub state: State,
}

impl Interpreter {
  pub fn new() -> Interpreter {
    Interpreter {
      state: State::new(),
    }
  }

  pub fn step(&mut self, mut e: Expr) -> Result<Expr> {
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
      Int(_) | Bool(_) | Func(_, _, _) | Undefined => {
        debug!("stepping on a value {:?}", e);
        return Err(RuntimeError::SteppingOnValue(e));
      },
      /**
       * Base cases
       */
      Uop(Not, ref e1) if e1.is_value() => {
        Bool(!e1.to_bool()?)
      },
      Uop(Neg, ref e1) if e1.is_value() => {
        Int(-1 * e1.to_int()?)
      },
      Bop(And, ref e1, ref e2) if e1.is_value() && e2.is_value() => {
        Bool(e1.to_bool()? && e2.to_bool()?)
      },
      Bop(Or, ref e1, ref e2) if e1.is_value() && e2.is_value() => {
        Bool(e1.to_bool()? || e2.to_bool()?)
      },
      Bop(Eq, ref e1, ref e2) if e1.is_value() && e2.is_value() => {
        Bool(*e1 == *e2)
      },
      Bop(Ne, ref e1, ref e2) if e1.is_value() && e2.is_value() => {
        Bool(*e1 != *e2)
      },
      Bop(Mod, ref e1, ref e2) if e1.is_value() && e2.is_value() => {
        let n1 = e1.to_int()?;
        let n2 = e2.to_int()?;

        // rust % gives the remainder, not modulus
        // let result = ((n1 % n2) + n2) % n2;
        let result = n1.checked_rem(n2)
          .ok_or(RuntimeError::IntegerUnderflow)?
          .checked_add(n2)
          .ok_or(RuntimeError::IntegerOverflow)?
          .checked_rem(n2)
          .ok_or(RuntimeError::IntegerUnderflow)?;

        Int(result)
      },
      Bop(Lt, ref e1, ref e2) if e1.is_value() && e2.is_value() => {
        Bool(e1.to_int()? < e2.to_int()?)
      },
      Bop(Gt, ref e1, ref e2) if e1.is_value() && e2.is_value() => {
        Bool(e1.to_int()? > e2.to_int()?)
      },
      Bop(Leq, ref e1, ref e2) if e1.is_value() && e2.is_value() => {
        Bool(e1.to_int()? <= e2.to_int()?)
      },
      Bop(Geq, ref e1, ref e2) if e1.is_value() && e2.is_value() => {
        Bool(e1.to_int()? >= e2.to_int()?)
      },
      Bop(Plus, ref e1, ref e2) if e1.is_value() && e2.is_value() => {
        match e1.to_int()?.checked_add(e2.to_int()?) {
          Some(n) => Int(n),
          None => return Err(RuntimeError::IntegerOverflow),
        }
      },
      Bop(Minus, ref e1, ref e2) if e1.is_value() && e2.is_value() => {
        match e1.to_int()?.checked_sub(e2.to_int()?) {
          Some(n) => Int(n),
          None => return Err(RuntimeError::IntegerUnderflow),
        }
      },
      Bop(Times, ref e1, ref e2) if e1.is_value() && e2.is_value() => {
        match e1.to_int()?.checked_mul(e2.to_int()?) {
          Some(n) => Int(n),
          None => return Err(RuntimeError::IntegerOverflow),
        }
      },
      Bop(Div, ref e1, ref e2) if e1.is_value() && e2.is_value() => {
        match e1.to_int()?.checked_div(e2.to_int()?) {
          Some(n) => Int(n),
          None => return Err(RuntimeError::IntegerUnderflow),
        }
      },
      Bop(Seq, ref v1, ref mut e2) if v1.is_value() => {
        *get_and_replace_mem!(e2)
      },
      Bop(Assign, ref v1, ref v2) if v1.is_var() && v2.is_value() => {
        let x = v1.to_var()?;
        self.state.assign(x, *v2.clone())?;
        debug!("done assigning {:?}", self.state.mem);
        *v2.clone()
      },
      Ternary(ref v1, ref e2, ref e3) if v1.is_value() => {
        match v1.to_bool()? {
          true => *e2.clone(),
          false => *e3.clone(),
        }
      },
      Decl(DConst, ref x, ref v1, ref mut e2) if v1.is_value() => {
        self.state.alloc_const(x.to_var()?, *v1.clone())?;
        *get_and_replace_mem!(e2)
      },
      Decl(DVar, ref x, ref v1, ref mut e2) if x.is_var() && v1.is_value() => {
        debug!("allocing {:?}", v1);
        self.state.alloc(x.to_var()?, *v1.clone())?;
        *get_and_replace_mem!(e2)
      },
      // lambda lift so we can use iter() in guard
      // https://github.com/rust-lang/rfcs/issues/1006
      FnCall(ref v1, ref es) if v1.is_func() && (|| es.iter().all(|v| v.is_value()))() => {
        match **v1 {
          Func(ref name, ref e1, ref xs) => {
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
      Scope(ref v1) if v1.is_value() => {
        self.state.end_scope();
        *v1.clone()
      },
      While(ref v1, ref e1o, _, ref e2o, ref e3) if v1.is_value() => {
        match v1.to_bool()? {
          true => While(
            Box::new(*e1o.clone()),
            e1o.clone(),
            e2o.clone(),
            e2o.clone(),
            e3.clone()
          ),
          false => *e3.clone(),
        }
      },
      Print(ref v1) if v1.is_value() => {
        println!("{}", v1);
        Expr::Undefined
      },
      /**
       * Search Cases
       */
      Bop(ref op, ref mut v1, ref mut e2) if v1.is_value() => {
        Bop(
          op.clone(),
          Box::new(*get_and_replace_mem!(v1)),
          Box::new(self.step(*get_and_replace_mem!(e2))?)
        )
      },
      Bop(Assign, ref mut v1, ref mut e2) if v1.is_var() => {
        Bop(
          Assign,
          Box::new(*get_and_replace_mem!(v1)),
          Box::new(self.step(*get_and_replace_mem!(e2))?)
        )
      },
      Bop(op, e1, e2) => {
        Bop(op, Box::new(self.step(*e1)?), e2)
      },
      Uop(op, e1) => {
        Uop(op, Box::new(self.step(*e1)?))
      },
      Ternary(e1, e2, e3) => {
        Ternary(Box::new(self.step(*e1)?), e2, e3)
      },
      While(ref mut e1, ref mut e1o, ref mut v2, ref mut e2o, ref mut e3) if v2.is_value() => {
        While(
          Box::new(
            self.step(*get_and_replace_mem!(e1))?
          ),
          get_and_replace_mem!(e1o),
          get_and_replace_mem!(v2),
          get_and_replace_mem!(e2o),
          get_and_replace_mem!(e3),
        )
      },
      While(e1, e1o, e2, e2o, e3) => {
        While(e1, e1o, Box::new(self.step(*e2)?), e2o, e3)
      },
      Decl(dt, addr, e1, e2) => {
        Decl(dt, addr, Box::new(self.step(*e1)?), e2)
      },
      FnCall(ref mut v1, ref args) if v1.is_func() => {
        let mut found_nonvalue = false;

        let stepped_args: Result<Vec<Expr>> = args.iter().map(|e| {
          if !found_nonvalue && !e.is_value() {
            found_nonvalue = true;
            self.step(e.clone())
          } else {
            Ok(e.clone())
          }
        }).collect();

        FnCall(get_and_replace_mem!(v1), stepped_args?)
      },
      FnCall(e1, args) => {
        FnCall(Box::new(self.step(*e1)?), args)
      },
      Scope(e1) => {
        Scope(Box::new(self.step(*e1)?))
      },
      Print(e1) => {
        Print(Box::new(self.step(*e1)?))
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