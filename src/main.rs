mod parser;

use parser::{Expr, Op};

fn main() {
  println!("foo")
}

#[test]
fn parse_expr_test() {
  // Parse basic numbers and var references
  assert_eq!(parser::parse_expr("1.1"), Ok(("", Expr::Float(1.1))));
  assert_eq!(parser::parse_expr("2.7"), Ok(("", Expr::Float(2.7))));
  assert_eq!(parser::parse_expr("hello"), Ok(("", Expr::Var("hello".to_string()))));
  assert_eq!(parser::parse_expr("foobar"), Ok(("", Expr::Var("foobar".to_string()))));

  // Parse a basic "Program"
  assert_eq!(parser::parse_program("foobar;1.3"), Ok(("", vec![Expr::Var("foobar".to_string()), Expr::Float(1.3)])));

  // Errors with basic parsing
  assert_eq!(parser::parse_program("1five"), Err(nom::Err::Error(nom::error::Error::new(&"five"[..], nom::error::ErrorKind::Eof))));
  assert_eq!(parser::parse_program("five 1"), Err(nom::Err::Error(nom::error::Error::new(&"1"[..], nom::error::ErrorKind::Eof))));

  // Parse basic arithmetic
  assert_eq!(parser::parse_program("five+1.4"), Ok(("", vec![Expr::BinOp(Op::Plus, Box::new(Expr::Var("five".to_string())), Box::new(Expr::Float(1.4)))])));
  assert_eq!(parser::parse_program("five + 1.4"), Ok(("", vec![Expr::BinOp(Op::Plus, Box::new(Expr::Var("five".to_string())), Box::new(Expr::Float(1.4)))])));
  assert_eq!(parser::parse_program("6 * 7"), Ok(("", vec![Expr::BinOp(Op::Multiply, Box::new(Expr::Float(6.0)), Box::new(Expr::Float(7.0)))])));

  // Parse arithmetic with precedence
  assert_eq!(parser::parse_program("5 + 6 * 7"), Ok(("", vec![Expr::BinOp(Op::Plus, Box::new(Expr::Float(5.0)), Box::new(Expr::BinOp(Op::Multiply, Box::new(Expr::Float(6.0)), Box::new(Expr::Float(7.0)))))])));
}

