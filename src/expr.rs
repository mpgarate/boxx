#[derive(Debug, PartialEq)] 
pub enum UnOp {
  Not,
  Neg,
}

#[derive(Debug, PartialEq)] 
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
}

#[derive(Debug, PartialEq)] 
pub enum Expr {
  Int(isize),
  Bool(bool),
  BinOp(BinOp, Box<Expr>, Box<Expr>),
  UnOp(UnOp, Box<Expr>),
}

fn to_int(e: Expr) -> isize {
  match e {
    Expr::Int(n) => n,
    _ => {
      debug!("cant turn into int: {:?}", e);
      panic!()
    }
  }
}

fn to_bool(e: Expr) -> bool {
  match e {
    Expr::Bool(b) => b,
    _ => {
      debug!("cant turn into bool: {:?}", e);
      panic!()
    }
  }
}

pub fn eval(e: Expr) -> Expr {
  match e {
    Expr::UnOp(UnOp::Not, e1) => {
      Expr::Bool(!to_bool(eval(*e1)))
    },
    Expr::UnOp(UnOp::Neg, e1) => {
      Expr::Int(-1 * to_int(eval(*e1)))
    },
    Expr::BinOp(BinOp::And, e1, e2) => {
      Expr::Bool(to_bool(eval(*e1)) && to_bool(eval(*e2)))
    },
    Expr::BinOp(BinOp::Or, e1, e2) => {
      Expr::Bool(to_bool(eval(*e1)) || to_bool(eval(*e2)))
    },
    Expr::BinOp(BinOp::Eq, e1, e2) => {
      Expr::Bool(eval(*e1) == eval(*e2))
    },
    Expr::BinOp(BinOp::Ne, e1, e2) => {
      Expr::Bool(eval(*e1) != eval(*e2))
    },
    Expr::BinOp(BinOp::Mod, e1, e2) => {
      let n1 = to_int(eval(*e1));
      let n2 = to_int(eval(*e2));

      // rust % gives the remainder, not modulus
      let result = ((n1 % n2) + n2) % n2;

      Expr::Int(result)
    },
    Expr::BinOp(BinOp::Lt, e1, e2) => {
      Expr::Bool(to_int(eval(*e1)) < to_int(eval(*e2)))
    },
    Expr::BinOp(BinOp::Gt, e1, e2) => {
      Expr::Bool(to_int(eval(*e1)) > to_int(eval(*e2)))
    },
    Expr::BinOp(BinOp::Leq, e1, e2) => {
      Expr::Bool(to_int(eval(*e1)) <= to_int(eval(*e2)))
    },
    Expr::BinOp(BinOp::Geq, e1, e2) => {
      Expr::Bool(to_int(eval(*e1)) >= to_int(eval(*e2)))
    },
    Expr::BinOp(BinOp::Plus, e1, e2) => {
      Expr::Int(to_int(eval(*e1)) + to_int(eval(*e2)))
    },
    Expr::BinOp(BinOp::Minus, e1, e2) => {
      Expr::Int(to_int(eval(*e1)) - to_int(eval(*e2)))
    },
    Expr::BinOp(BinOp::Times, e1, e2) => {
      Expr::Int(to_int(eval(*e1)) * to_int(eval(*e2)))
    },
    Expr::BinOp(BinOp::Div, e1, e2) => {
      Expr::Int(to_int(eval(*e1)) / to_int(eval(*e2)))
    },
    Expr::BinOp(BinOp::Seq, e1, e2) => {
      eval(*e1);
      eval(*e2)
    },
    Expr::Int(_) => e,
    Expr::Bool(_) => e,
  }
}
