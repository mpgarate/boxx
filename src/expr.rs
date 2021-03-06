use std::fmt;

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

#[derive(Clone, Debug, PartialEq)] 
pub enum Val {
  Int(isize),
  Bool(bool),
  Undefined,
  Func(Option<Box<Expr>>, Box<Expr>, Vec<Expr>),
}

#[derive(Clone, Debug, PartialEq)] 
pub enum Expr {
  Val(Val),
  Var(String),
  Bop(BinOp, Box<Expr>, Box<Expr>),
  Uop(UnOp, Box<Expr>),
  Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
  While(Box<Expr>, Box<Expr>, Box<Expr>, Box<Expr>, Box<Expr>),
  Decl(Dec, Box<Expr>, Box<Expr>, Box<Expr>),
  FnCall(Box<Expr>, Vec<Expr>),
  Scope(Box<Expr>),
  Print(Box<Expr>),
}

impl fmt::Display for Val {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      Val::Int(n) => write!(f, "{}", n),
      Val::Bool(b) => write!(f, "{}", b),
      _ => write!(f, "cannot print this thing")
    }
  }
}
