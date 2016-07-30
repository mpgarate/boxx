extern crate boxx;

use boxx::expr::{evaluate};
use boxx::ast::{State, Expr};

use std::io::{Write, stdout, stdin};

fn main() {
  let mut state = State::from(Expr::Int(0));

  loop {
    print!("boxx> ");
    let _ = stdout().flush();

    let mut input = String::new();
    match stdin().read_line(&mut input) {
      Ok(_) => {
        if input == "exit\n".to_string() { 
          break;
        }
        println!("{:?}", evaluate(&input, &mut state))
      },
      Err(e) => print!("error: {}", e)
    }
    let _ = stdout().flush();
  }
}

