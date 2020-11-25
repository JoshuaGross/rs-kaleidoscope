mod parser;

use parser::Expr;

fn main() {
  println!("foo")
}

#[test]
fn parse_expr_test() {
  assert_eq!(parser::parse_expr("1.1"), Ok(("", Expr::Float(1.1))));
  assert_eq!(parser::parse_expr("2.7"), Ok(("", Expr::Float(2.7))));
  assert_eq!(parser::parse_expr("hello"), Ok(("", Expr::Var("hello".to_string()))));
  assert_eq!(parser::parse_expr("foobar"), Ok(("", Expr::Var("foobar".to_string()))));

  // TODO: figure out how to test errors
  //assert_eq!(parse_expr("1five"), Err(nom::Err::Error("five", nom::Err::Eof)));
  assert_eq!(parser::parse_expr("1five"), Err(nom::Err::Error(nom::error::Error::new(&"five"[..], nom::error::ErrorKind::Eof))));
  assert_eq!(parser::parse_expr("five 1"), Err(nom::Err::Error(nom::error::Error::new(&" 1"[..], nom::error::ErrorKind::Eof))));
}

