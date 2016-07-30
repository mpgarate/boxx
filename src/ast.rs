use ast::Expr::*;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)] 
pub enum UnOp {
  Not,
  Neg,
}

#[derive(Clone, Debug, PartialEq)] 
pub enum BinOp {
  Plus,
  Minus,
  Times,
  Div,
  Eq,
  Ne,
  Leq,
  Geq,
  Lt,
  Gt,
  And,
  Or,
  Mod,
  Seq,
  Assign,
}

#[derive(Clone, Debug, PartialEq)] 
pub enum Dec {
  DVar,
  DConst
}

#[derive(Clone, Debug)] 
pub struct State {
  pub mem: Vec<HashMap<String, Expr>>,
  pub expr: Expr,
}

impl State {
  pub fn from(e: Expr) -> State {
    return State {
      mem: vec!(HashMap::new()),
      expr: e,
    }
  }

  pub fn begin_scope(&mut self) {
    self.mem.push(HashMap::new());
  }

  pub fn end_scope(&mut self) {
    self.mem.pop();
  }

  pub fn with(&mut self, e1: Expr) -> &mut State {
    self.expr = e1;
    return self;
  }

  pub fn alloc(&mut self, s: String, v1: Expr) {
    self.mem.last_mut().unwrap().insert(s, v1);
  }

  pub fn assign(&mut self, s: String, v1: Expr) {
    match self.mem.iter_mut().rev().find(|m| m.contains_key(&s)) {
      Some(m) => m.insert(s, v1),
      None => {
        debug!("cannot assign; no value for s {:?}", s);
        panic!("cannot assign; no value for s")
      }
    };
  }

  pub fn get(&mut self, s: String) -> Expr {
    match self.mem.iter().rev().find(|m| m.contains_key(&s)) {
      Some(m) => m.get(&s).unwrap().clone(),
      None => {
        debug!("no value for s{:?}", s);
        panic!("no value for s")
      }
    }
  }
}

#[derive(Clone, Debug, PartialEq)] 
pub enum Expr {
  Int(isize),
  Bool(bool),
  Var(String),
  Bop(BinOp, Box<Expr>, Box<Expr>),
  Uop(UnOp, Box<Expr>),
  Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
  Decl(Dec, Box<Expr>, Box<Expr>, Box<Expr>),
  Func(Option<Box<Expr>>, Box<Expr>, Vec<Expr>),
  FnCall(Box<Expr>, Vec<Expr>),
  Addr(String),
}

impl Expr {
  pub fn is_int(&self) -> bool {
    match *self {
      Int(_) => true,
      _ => false,
    }
  }

  pub fn is_bool(&self) -> bool {
    match *self {
      Bool(_) => true,
      _ => false,
    }
  }

  pub fn is_func(&self) -> bool {
    match *self {
      Func(_, _, _) => true,
      _ => false,
    }
  }

  pub fn is_value(&self) -> bool {
    match *self {
      Int(_) | Bool(_) | Var(_) | Func(_, _, _) => true,
      _ => false,
    }
  }

  pub fn is_addr(&self) -> bool {
    match *self {
      Addr(_) => true,
      _ => false,
    }
  }

  pub fn to_int(&self) -> isize {
    match *self {
      Int(n) => n,
      _ => {
        debug!("cant turn into int: {:?}", self);
        panic!()
      }
    }
  }

  pub fn to_bool(&self) -> bool {
    match *self {
      Bool(b) => b,
      _ => {
        debug!("cant turn into bool: {:?}", self);
        panic!()
      }
    }
  }

  pub fn to_addr(&self) -> String {
    match self.clone() {
      Addr(a) => a,
      Var(s) => s,
      _ => {
        debug!("cant turn into addr: {:?}", self);
        panic!()
      }
    }
  }
}

