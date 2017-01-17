extern crate boxx;

#[cfg(test)]
mod test {
  use boxx::parser::parser::{parse};
  use boxx::expr::{Val, Expr, BinOp};
  extern crate env_logger;

  #[test]
  fn test_mult_div() {
    assert_eq!(
      Expr::Bop(
        BinOp::Times,
        Box::new(Expr::Val(Val::Int(3))),
        Box::new(Expr::Val(Val::Int(4))),
      ),
      parse("3*4").unwrap()
    );

    assert_eq!(
      Expr::Bop(
        BinOp::Div,
        Box::new(Expr::Val(Val::Int(3))),
        Box::new(Expr::Val(Val::Int(4))),
      ),
      parse("3/4").unwrap()
    );
  }

  #[test]
  fn test_parse_add_subtract_parens() {
    assert_eq!(
      Expr::Bop(
        BinOp::Plus,
        Box::new(Expr::Val(Val::Int(3))),
        Box::new(Expr::Val(Val::Int(4))),
      ),
      parse("3+4").unwrap()
    );

    assert_eq!(
      Expr::Bop(
        BinOp::Plus,
        Box::new(
          Expr::Bop(
            BinOp::Plus,
            Box::new(Expr::Val(Val::Int(3))),
            Box::new(Expr::Val(Val::Int(4))),
          ),
        ),
        Box::new(Expr::Val(Val::Int(5))),
      ),
      parse("3+4+5").unwrap()
    );

    assert_eq!(
      Expr::Bop(BinOp::Plus,
        Box::new(Expr::Val(Val::Int(3))),
        Box::new(
          Expr::Bop(
            BinOp::Plus,
            Box::new(Expr::Val(Val::Int(4))),
            Box::new(Expr::Val(Val::Int(5))),
          ),
        ),
      ),
      parse("3+(4+5)").unwrap()
    );

    assert_eq!(
      Expr::Bop(
        BinOp::Minus,
        Box::new(Expr::Val(Val::Int(3))),
        Box::new(Expr::Val(Val::Int(4))),
      ),
      parse("3-4").unwrap()
    );

    assert_eq!(
      Expr::Bop(
        BinOp::Minus,
        Box::new(
          Expr::Bop(
            BinOp::Minus,
            Box::new(Expr::Val(Val::Int(3))),
            Box::new(Expr::Val(Val::Int(4))),
          ),
        ),
        Box::new(Expr::Val(Val::Int(5))),
      ),
      parse("3-4-5").unwrap()
    );

    assert_eq!(
      Expr::Bop(
        BinOp::Minus,
        Box::new(Expr::Val(Val::Int(3))),
        Box::new(
          Expr::Bop(
            BinOp::Minus,
            Box::new(Expr::Val(Val::Int(4))),
            Box::new(Expr::Val(Val::Int(5))),
          ),
        ),
      ),
      parse("3-(4-5)").unwrap()
    );

    assert_eq!(
      Expr::Bop(
        BinOp::Minus,
        Box::new(
          Expr::Bop(
            BinOp::Plus,
            Box::new(Expr::Val(Val::Int(4))),
            Box::new(Expr::Val(Val::Int(7))),
          ),
        ),
        Box::new(Expr::Val(Val::Int(3))),
      ),
      parse("(4+7)-3").unwrap()
    );
  }
}
