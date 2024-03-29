extern crate boxx;

#[cfg(test)]
mod tests {
  extern crate boxx;
  use boxx::interpreter::Interpreter;
  use boxx::expr::{Val, Expr};
  use boxx::runtime_error::RuntimeError;

  extern crate env_logger;

  fn boxx(input: &str) -> Result<Expr, RuntimeError> {
    let mut interpreter = Interpreter::new();
    interpreter.eval(input)
  }

  #[test]
  pub fn test_parser_error() {
    let _ = env_logger::init();

    // TODO: consider expecting an error for this
    // assert_eq!(Ok(Expr::Val(Val::Int(2)), boxx("1 +"));
  }

  #[test]
  pub fn test_interpreter() {
    let _ = env_logger::init();

    let mut interpreter = Interpreter::new();

    assert_eq!(
      Expr::Val(Val::Int(2)),
      interpreter.eval("1 + 1").unwrap()
    );

    assert_eq!(
      Expr::Val(Val::Undefined),
      interpreter.eval("var x = 3;").unwrap()
    );

    assert_eq!(
      Expr::Val(Val::Int(3)),
      interpreter.eval("x").unwrap()
    );


    assert_eq!(
      Expr::Val(Val::Undefined),
      interpreter.eval("fn double(x) { x + x };").unwrap()
    );

    assert_eq!(
      Expr::Val(Val::Int(48)),
      interpreter.eval("double(24)").unwrap()
    );
  }

  #[test]
  pub fn test_integer_overflow() {
    let _ = env_logger::init();

    let max_int = isize::max_value();
    let min_int = isize::min_value();

    assert_eq!(
      Ok(Expr::Val(Val::Int(max_int))),
      boxx(format!("
        {}
      ", max_int).as_str())
    );

    assert_eq!(
      Err(RuntimeError::IntegerOverflow),
      boxx(format!("
        {} + 1
      ", max_int).as_str())
    );

    assert_eq!(
      Err(RuntimeError::IntegerOverflow),
      boxx(format!("
        {} * 2
      ", max_int).as_str())
    );

    // we have to add 1 in order to parse successfully, since it is parsed in
    // as a positive integer and then negated in the interpreter
    assert_eq!(
      Err(RuntimeError::IntegerUnderflow),
      boxx(format!("
        {} - 2
      ", min_int + 1).as_str())
    );

    assert_eq!(
      Err(RuntimeError::IntegerUnderflow),
      boxx(format!("
        ({} - 1) / -1
      ", min_int + 1).as_str())
    );

    assert_eq!(
      Err(RuntimeError::IntegerUnderflow),
      boxx("1 % 0")
    );
  }

  #[test]
  pub fn test_print() {
    let _ = env_logger::init();

    // TODO: it would be nice to test the stdout for this
    assert_eq!(
      Ok(Expr::Val(Val::Undefined)),
      boxx("
        var x = 555;
        print(x);
      ")
    );
  }

  #[test]
  pub fn test_comments() {
    let _ = env_logger::init();

    assert_eq!(
      Ok(Expr::Val(Val::Int(3))),
      boxx("
        var i = 0;
        i = i + 2; // adding two
        i = i + 1; // adding one
        i
      ")
    );

    /*
     * TODO: block comments
    assert_eq!(
      Ok(Expr::Val(Val::Int(2))),
      boxx("
        var i = 0;
        i = i + 2;
        /* not doing this i = i + 1; */
        i
      ")
    );
    */
  }

  #[test]
  pub fn test_while_loop() {
    let _ = env_logger::init();

    assert_eq!(
      Ok(Expr::Val(Val::Int(12))),
      boxx("
        var i = 0;

        while (i < 10) {
         if (i % 2 == 0) {
           i = i + 1
         } else {
           i = i + 3
         }
        };
        i
      ")
    );

    assert_eq!(Ok(Expr::Val(Val::Int(11))), boxx("var i = 1; while (i < 11) { i = i + 1; i }; i"));
    assert_eq!(Ok(Expr::Val(Val::Int(10))), boxx("var i = 1; var x = 4; while (i % 2 != 0) { i = i + x; x = x + 1; x }; i"));
    assert_eq!(
      Ok(Expr::Val(Val::Int(96))),
      boxx("
        fn foo(x) { x * 2 };
        var x = 3;
        while (foo(x) < 100) {
          x = foo(x)
        };
        x
      ")
    );

    assert_eq!(
      Ok(Expr::Val(Val::Int(96))),
      boxx("
        fn foo(x) { x * 2 };
        var x = 3;
        while ((x = foo(x)) < 96) { 0 };
        x
      ")
    );

    assert_eq!(
      Ok(Expr::Val(Val::Int(16))),
      boxx("
        fn foo(x) { x + 1 };
        var x = 1;
        while (x < 10) {
          x = foo(x);
          x = 2 * foo(x);
          x + 1
        };
        x
      ")
    );
  }

  #[test]
  pub fn test_undefined() {
    let _ = env_logger::init();

    assert_eq!(Ok(Expr::Val(Val::Undefined)), boxx("var x = 2;"));

    assert_eq!(
      Ok(Expr::Val(Val::Int(8))),
      boxx("
        var x = 4;
        var foo = fn(z) {
          x = z + 2;
        };
        foo(x);
        foo(x);
        x
      ")
    );
  }

  #[test]
  pub fn test_mut_var() {
    let _ = env_logger::init();

    assert_eq!(Ok(Expr::Val(Val::Int(555))), boxx("var x = 55; var y = 500; x + y"));

    assert_eq!(
      Ok(Expr::Val(Val::Int(2))),
      boxx("var x = 1; var y = 2; x = y; y = 3; x")
    );

    assert_eq!(Ok(Expr::Val(Val::Int(2))), boxx("var x = 1; x = 2; x"));

    assert_eq!(
      Ok(Expr::Val(Val::Int(5))),
      boxx("var x = 3; var y = 2; x = y; y = x; let z = 1; z + x + y")
    );

    assert_eq!(
      Ok(Expr::Val(Val::Int(20))),
      boxx("
        var x = 4;
        fn foo(z) {
          x * z 
        };
        foo(x) + x
      ")
    );

    assert_eq!(
      Ok(Expr::Val(Val::Int(15))),
      boxx("var x = 4; fn foo(z) { var x = 7; x + z }; foo(x) + x")
    );

    /*
       TODO: allow var bindings so that fn params can be reassigned
    assert_eq!(
      Ok(Expr::Val(Val::Int(23))),
      boxx("var x = 4; fn foo(z) { var x = 7; z = x; x = 12; x + z }; foo(x) + x")
    );
    */

    assert_eq!(Ok(Expr::Val(Val::Int(2))), boxx("var i = 1; i = i + 1; i"));

    assert_eq!(
      Ok(Expr::Val(Val::Int(13))),
      boxx("var x = 10; var foo = fn(x) { var foo = fn (y) { var x = 3; y + x }; foo(x) }; foo(x) ")
    );

    assert_eq!(Ok(Expr::Val(Val::Int(5))), boxx("var x = 3; x = fn() { 4 + 1 }; x()"));
    assert_eq!(Ok(Expr::Val(Val::Int(3))), boxx("var x = fn() { 4 + 1 }; x = 3; x"));
  }


  #[test]
  pub fn test_if_else() {
    let _ = env_logger::init();

    assert_eq!(
      Ok(Expr::Val(Val::Int(999))),
      boxx("var b = 1; if (true) { b = 999; }; b ")
    );

    assert_eq!(
      Ok(Expr::Val(Val::Int(34))),
      boxx("if (true && false) { 32 } else if (!true && true) { 33 } else { 34 }")
    );

    assert_eq!(
      Ok(Expr::Val(Val::Int(32))),
      boxx("if (true || false) { 32 } else if (!true && true) { 33 } else { 34 }")
    );

    assert_eq!(
      Ok(Expr::Val(Val::Int(30))),
      boxx("if (true && false) { 32 } else { 30 }")
    );

    assert_eq!(
      Ok(Expr::Val(Val::Int(52))),
      boxx("if (let x = 4; x > 3) { 52 } else { 30 }")
    );

    assert_eq!(
      Ok(Expr::Val(Val::Int(22))),
      boxx("if (true) { 11 } else { 0 }; 22")
    );
  }

  #[test]
  pub fn test_func() {
    let _ = env_logger::init();

    assert_eq!(
      Ok(Expr::Val(Val::Int(8))),
      boxx("
        var x = 4;
        fn foo(z) {
          x = z + 2;
        };
        foo(x);
        foo(x);
        x
      ")
    );

    assert_eq!(Ok(Expr::Val(Val::Int(2))), boxx("let x = 4; fn foo() { let x = 1; x + 1 }; foo()"));
    assert_eq!(Ok(Expr::Val(Val::Int(6))), boxx("let x = 5; fn foo() { x + 1 }; foo()"));
    assert_eq!(Ok(Expr::Val(Val::Int(60))), boxx("fn foo() { 5 }; fn bar() { fn foo() { 6 }; foo() * 10 }; bar()"));
    assert_eq!(Ok(Expr::Val(Val::Int(50))), boxx("fn foo() { 5 }; fn bar() { foo() * 10 }; bar()"));

    assert_eq!(Ok(Expr::Val(Val::Int(12))), boxx("fn sum(a, b) { a + b }; sum(sum(3, 4), 5)"));
    assert_eq!(Ok(Expr::Val(Val::Int(12))), boxx("fn tx_two(a) { 2 * a }; tx_two(tx_two(3))"));

    assert_eq!(
      Ok(Expr::Val(Val::Int(41))),
      boxx("
        fn foo(a) {
          a < 40 ? foo(a + 3) : a
        };

        foo(20)
      ")
    );

    assert_eq!(
      Ok(Expr::Val(Val::Int(21))),
      boxx("
        fn fib(n) {
          n == 0 ? 0 : (n == 1 ? 1 : fib(n - 1) + fib(n - 2))
        };

        fib(8)
      ")
    );

    assert_eq!(
      Ok(Expr::Val(Val::Int(21))),
      boxx("
        var fib = fn(n) {
          n == 0 ? 0 : (n == 1 ? 1 : fib(n - 1) + fib(n - 2))
        };

        fib(8)
      ")
    );

    assert_eq!(
      Ok(Expr::Val(Val::Int(28))),
      boxx("
        fn foo(a) {
          1 + a
        };

        fn bar(b) {
          5 * b
        };
        
        foo(bar(4)) + 7
      ")
    );

    assert_eq!(Ok(Expr::Val(Val::Int(12))), boxx("fn b() { 5 + 5 }; let a = b; a() + 2"));
    assert_eq!(Ok(Expr::Val(Val::Int(12))), boxx("let b = fn() { 5 + 5 }; let a = b; a() + 2"));
    assert_eq!(Ok(Expr::Val(Val::Int(12))), boxx("fn foo(a) { 1 + a }; foo(4) + 7"));
    assert_eq!(Ok(Expr::Val(Val::Int(12))), boxx("let foo = fn(a) { 1 + a }; foo(4) + 7"));

    assert_eq!(Ok(Expr::Val(Val::Int(2))), boxx("fn foo() { 1 + 1 }; foo()"));
    assert_eq!(Ok(Expr::Val(Val::Int(7))), boxx("fn foo() { 1 + 3 }; foo() + 3"));
    assert_eq!(Ok(Expr::Val(Val::Int(9))), boxx("fn foo() { 1 + 3 }; fn bar() { foo() + 1}; 4 + bar()"));

    assert_eq!(Ok(Expr::Val(Val::Int(2))), boxx("let foo = fn() { 1 + 1 }; foo()"));
    assert_eq!(Ok(Expr::Val(Val::Int(7))), boxx("let foo = fn() { 1 + 3 }; foo() + 3"));
    assert_eq!(Ok(Expr::Val(Val::Int(9))), boxx("let foo = fn() { 1 + 3 }; let bar = fn() { foo() + 1}; 4 + bar()"));

    assert_eq!(Ok(Expr::Val(Val::Int(4))), boxx("fn() { 1 + 3 }()"));
    assert_eq!(Ok(Expr::Val(Val::Int(4))), boxx("let foo = fn() { 1 + 3 }(); foo"));
  }

  #[test]
  pub fn test_const_decl() {
    let _ = env_logger::init();
    assert_eq!(Ok(Expr::Val(Val::Int(3))), boxx("let x = 1 + 2; x"));
    assert_eq!(Ok(Expr::Val(Val::Int(1))), boxx("let x = 1; x"));
    assert_eq!(Ok(Expr::Val(Val::Int(8))), boxx("let x = 5; let y = 3; let z = x + y; z"));

    assert_eq!(Ok(Expr::Val(Val::Int(3))), boxx("let x = (1 > 2) ? 0 : 3; x"));

    // using let keyword again re-binds value
    assert_eq!(Ok(Expr::Val(Val::Int(5))), boxx("let x = 2; let x = 3; x + 2"));
    assert_eq!(Ok(Expr::Val(Val::Int(52))), boxx("let underscore_name = 51; 1 + underscore_name"));
  }

  #[test]
  pub fn test_ternary() {
    let _ = env_logger::init();
    assert_eq!(Ok(Expr::Val(Val::Int(1))), boxx("true ? 1 : 0"));
    assert_eq!(Ok(Expr::Val(Val::Int(0))), boxx("false ? 1 : 0"));
    assert_eq!(Ok(Expr::Val(Val::Int(3))), boxx("(false ? 1 : 0); 1 + 2"));
    assert_eq!(Ok(Expr::Val(Val::Int(3))), boxx("false ? 1 : 0; 1 + 2"));
    assert_eq!(Ok(Expr::Val(Val::Int(0))), boxx("((1 + 1) > 3) ? 1 : 0"));
    assert_eq!(Ok(Expr::Val(Val::Int(14))), boxx("((1 + 1) > 3) ? true && false : 12 + 2"));
    assert_eq!(Ok(Expr::Val(Val::Int(14))), boxx("1 + 1 > 3 ? true && false : 12 + 2"));
    assert_eq!(
      Ok(Expr::Val(Val::Int(10))),
      boxx(
          "(false || true) ? ((1 + 2 > 12) ? 9 : 10) : ((1 + 2 < 12) ? 6 : 7)"
       )
    );
    // same as above but without parens
    assert_eq!(
      Ok(Expr::Val(Val::Int(10))),
      boxx(
          "false || true ? 1 + 2 > 12 ? 9 : 10 : 1 + 2 < 12 ? 6 : 7"
       )
    );

    assert_eq!(Ok(Expr::Val(Val::Bool(true))), boxx("1 + 2 > (1 == 0 ? 5 : 1)"));

    assert_eq!(Ok(Expr::Val(Val::Int(-1))), boxx("true;false ? 1;2 : 0;-1"));
  }

  #[test]
  pub fn test_seq() {
    let _ = env_logger::init();
    assert_eq!(Ok(Expr::Val(Val::Int(5))), boxx("3;5"));
    assert_eq!(Ok(Expr::Val(Val::Int(4))), boxx("let x = 3;let y = 1;x + y"));
  }

  #[test]
  pub fn test_mod() {
    assert_eq!(Ok(Expr::Val(Val::Int(0))), boxx("1 % 1"));
    assert_eq!(Ok(Expr::Val(Val::Int(2))), boxx("7 % 5"));
    assert_eq!(Ok(Expr::Val(Val::Int(3))), boxx("-7 % 5"));
    assert_eq!(Ok(Expr::Val(Val::Int(-2))), boxx("-7 % -5"));
  }

  #[test]
  pub fn test_or_and_and() {
    assert_eq!(Ok(Expr::Val(Val::Bool(true))), boxx("true && true"));
    assert_eq!(Ok(Expr::Val(Val::Bool(false))), boxx("false && false"));
    assert_eq!(Ok(Expr::Val(Val::Bool(false))), boxx("true && false"));
    assert_eq!(Ok(Expr::Val(Val::Bool(false))), boxx("false && true"));

    assert_eq!(Ok(Expr::Val(Val::Bool(true))), boxx("true || true"));
    assert_eq!(Ok(Expr::Val(Val::Bool(false))), boxx("false || false"));
    assert_eq!(Ok(Expr::Val(Val::Bool(true))), boxx("true || false"));
    assert_eq!(Ok(Expr::Val(Val::Bool(true))), boxx("false || true"));
  }


  #[test]
  pub fn test_not_and_neg() {
    let _ = env_logger::init();

    assert_eq!(Ok(Expr::Val(Val::Bool(true))), boxx("!true || true"));

    assert_eq!(Ok(Expr::Val(Val::Int(0))), boxx("-1 * -1 + -1"));

    assert_eq!(Ok(Expr::Val(Val::Bool(true))), boxx("!false"));

    assert_eq!(Ok(Expr::Val(Val::Bool(true))), boxx("!(true == false)"));
    assert_eq!(Ok(Expr::Val(Val::Bool(true))), boxx("!((1 == 1) == (3 <= 2))"));
    assert_eq!(Ok(Expr::Val(Val::Bool(false))), boxx("!((1 == 1) == !(3 <= 2))"));
    assert_eq!(Ok(Expr::Val(Val::Bool(true))), boxx("!!(!(!(true)))"));

    assert_eq!(Ok(Expr::Val(Val::Int(-1))), boxx("-1"));
    assert_eq!(Ok(Expr::Val(Val::Int(-100))), boxx("-(20 * 5)"));
    assert_eq!(Ok(Expr::Val(Val::Int(-100))), boxx("-(-20 * -5)"));
    assert_eq!(Ok(Expr::Val(Val::Int(-100))), boxx("(20 * -5)"));
    assert_eq!(Ok(Expr::Val(Val::Int(-100))), boxx("(-20 * 5)"));
    assert_eq!(Ok(Expr::Val(Val::Int(100))), boxx("(-20 * -5)"));
    assert_eq!(Ok(Expr::Val(Val::Int(100))), boxx("-(20 * -5)"));
    assert_eq!(Ok(Expr::Val(Val::Int(100))), boxx("-(-20 * 5)"));
    assert_eq!(Ok(Expr::Val(Val::Int(0))), boxx("1 + -1"));
    assert_eq!(Ok(Expr::Val(Val::Int(2))), boxx("1 - -1"));
    assert_eq!(Ok(Expr::Val(Val::Int(0))), boxx("-1 - -1"));
    assert_eq!(Ok(Expr::Val(Val::Int(-2))), boxx("-1 - 1"));
    assert_eq!(Ok(Expr::Val(Val::Int(-2))), boxx("-1 * 2"));
    assert_eq!(Ok(Expr::Val(Val::Int(-2))), boxx("2 * -1"));
    assert_eq!(Ok(Expr::Val(Val::Int(-2))), boxx("-2 * 1"));
    assert_eq!(Ok(Expr::Val(Val::Int(-1))), boxx("-(2 * 1) + 1"));
    assert_eq!(Ok(Expr::Val(Val::Int(1))), boxx("(2 * 1) + -1"));
  }

  #[test]
  pub fn test_comparison_operators() {
    assert_eq!(Ok(Expr::Val(Val::Bool(true))), boxx("1 == 1"));
    assert_eq!(Ok(Expr::Val(Val::Bool(false))), boxx("1 == 2"));
    assert_eq!(Ok(Expr::Val(Val::Bool(false))), boxx("(1 == 1) == (1 == 2)"));
    assert_eq!(Ok(Expr::Val(Val::Bool(true))), boxx("(5 == 2) == (1 == 2)"));
    assert_eq!(Ok(Expr::Val(Val::Bool(true))), boxx("(6 == 6) == true"));
    assert_eq!(Ok(Expr::Val(Val::Bool(false))), boxx("1 == true"));
    assert_eq!(Ok(Expr::Val(Val::Bool(true))), boxx("false == false"));

    assert_eq!(Ok(Expr::Val(Val::Bool(true))), boxx("1 > 0"));
    assert_eq!(Ok(Expr::Val(Val::Bool(false))), boxx("1 < 0"));

    assert_eq!(Ok(Expr::Val(Val::Bool(true))), boxx("88 > 34"));
    assert_eq!(Ok(Expr::Val(Val::Bool(false))), boxx("1 < 1"));
    assert_eq!(Ok(Expr::Val(Val::Bool(false))), boxx("1 > 1"));

    assert_eq!(Ok(Expr::Val(Val::Bool(true))), boxx("88 != 34"));
    assert_eq!(Ok(Expr::Val(Val::Bool(false))), boxx("88 != 88"));
    assert_eq!(Ok(Expr::Val(Val::Bool(true))), boxx("88 <= 88"));
    assert_eq!(Ok(Expr::Val(Val::Bool(true))), boxx("88 >= 88"));
    assert_eq!(Ok(Expr::Val(Val::Bool(true))), boxx("1 >= 0"));
    assert_eq!(Ok(Expr::Val(Val::Bool(false))), boxx("1 >= 12"));

    assert_eq!(Ok(Expr::Val(Val::Bool(false))), boxx("true != true"));
    assert_eq!(Ok(Expr::Val(Val::Bool(true))), boxx("true != false"));
  }

  #[test]
  pub fn test_spaces() {
    assert_eq!(Ok(Expr::Val(Val::Int(2))), boxx("1 + 1"));
    assert_eq!(Ok(Expr::Val(Val::Int(12))), boxx(" (3+   3)* 2      "));
    assert_eq!(Ok(Expr::Val(Val::Int(7))), boxx("1 + 3*(3 + (1 - 2))"));
  }

  #[test]
  pub fn test_eval_mult() {
    assert_eq!(Ok(Expr::Val(Val::Int(12))), boxx("6*2"));
    assert_eq!(Ok(Expr::Val(Val::Int(12))), boxx("(3+3)*2"));
    assert_eq!(Ok(Expr::Val(Val::Int(0))), boxx("(3+3)*0"));
  }

  #[test]
  pub fn test_eval_div() {
    assert_eq!(Ok(Expr::Val(Val::Int(6))), boxx("12/2"));
    //assert_eq!(Ok(Expr::Float(1.5)), boxx("3/2"));
  }

  #[test]
  pub fn test_eval_addition() {
    assert_eq!(Ok(Expr::Val(Val::Int(3))), boxx("1+2"));
    assert_eq!(Ok(Expr::Val(Val::Int(16))), boxx("5+7+4"));
    assert_eq!(Ok(Expr::Val(Val::Int(-1))), boxx("1-2"));
    assert_eq!(Ok(Expr::Val(Val::Int(-100))), boxx("32-132"));
    assert_eq!(Ok(Expr::Val(Val::Int(-120))), boxx("32-132-20"));

    assert_eq!(Ok(Expr::Val(Val::Int(-80))), boxx("32-(132-20)"));

    assert_eq!(Ok(Expr::Val(Val::Int(-6))), boxx("4-(7+3)"));
    assert_eq!(Ok(Expr::Val(Val::Int(0))), boxx("4-(7-3)"));
    assert_eq!(Ok(Expr::Val(Val::Int(8))), boxx("4+(7-3)"));
    assert_eq!(Ok(Expr::Val(Val::Int(8))), boxx("(4+7)-3)"));
    assert_eq!(Ok(Expr::Val(Val::Int(0))), boxx("(4-7)+3)"));
    assert_eq!(Ok(Expr::Val(Val::Int(14))), boxx("(4+7)+3)"));

    assert_eq!(Ok(Expr::Val(Val::Int(2))), boxx("(1-1)+(2-2)+(3-3)+((1+2)-((3-2)+1)+1)"));
    assert_eq!(Ok(Expr::Val(Val::Int(0))), boxx("((((((((((1-1)))+1))))-1)))"));
  }
}
