extern crate nom;
use nom::{
  branch::alt,
  bytes::complete::is_a,
  character::complete::one_of,
  combinator::recognize,
  sequence::preceded,
  number::complete,
  IResult
};


type Name = String;

#[derive(Debug,PartialEq)]
enum Expr {
  Float(f32),
  BinOp(Op, Box<Expr>, Box<Expr>),
  Var(Name),
  Call(Name, ExprList),
  Function(Name, ExprList, Box<Expr>),
  Extern(Name, ExprList)
}

#[derive(Debug,PartialEq)]
enum ExprList {
  Cons(Box<Expr>, Box<ExprList>),
  Nil
}

#[derive(Debug,PartialEq)]
enum Op {
  Plus,
  Minus,
  Multiply,
  Divide
}

fn parse_float(input: &str) -> IResult<&str, Expr> {
  // FIXME: error handling?
  let (input, num) = complete::float(input)?;

  Ok((input, Expr::Float(num)))
}

fn parse_var(input: &str) -> IResult<&str, Expr> {
  let initial_chars: &str = "_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
  let remaining_chars: &str = "_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

  // Returns whole strings matched by the given parser.
  let (input, ident) = recognize(
    // Runs the first parser, if succeeded then runs second, and returns the second result.
    // Note that returned ok value of `preceded()` is ignored by `recognize()`.
    preceded(
      // Parses a single character contained in the given string.
      one_of(initial_chars),
      // Parses the longest slice consisting of the given characters
      is_a(remaining_chars),
    )
  )(input)?;

  Ok((input, Expr::Var(ident.to_string())))
}

fn parse_expr(input: &str) -> IResult<&str, Expr> {
  return nom::combinator::all_consuming(alt((parse_float, parse_var)))(input);
}

fn main() {
  println!("foo")
}

#[test]
fn parse_expr_test() {
  assert_eq!(parse_expr("1.1"), Ok(("", Expr::Float(1.1))));
  assert_eq!(parse_expr("2.7"), Ok(("", Expr::Float(2.7))));
  assert_eq!(parse_expr("hello"), Ok(("", Expr::Var("hello".to_string()))));
  assert_eq!(parse_expr("foobar"), Ok(("", Expr::Var("foobar".to_string()))));

  // TODO: figure out how to test errors
  //assert_eq!(parse_expr("1five"), Err(nom::Err::Error("five", nom::Err::Eof)));
  assert_eq!(parse_expr("1five"), Err(nom::Err::Error(nom::error::Error::new(&"five"[..], nom::error::ErrorKind::Eof))));
}

