extern crate nom;

use nom::{
  branch::alt,
  bytes::complete::{is_a, tag},
  character::complete::{char, one_of},
  combinator::{map, recognize},
  sequence::preceded,
  multi::separated_list1,
  number::complete,
  IResult
};

type Name = String;

#[derive(Debug,PartialEq)]
pub enum Expr {
  Float(f32),
  Var(Name),
  BinOp(Op, Box<Expr>, Box<Expr>),
  /*Call(Name, ExprList),
  Function(Name, ExprList, Box<Expr>),
  Extern(Name, ExprList)*/
}


pub type Program = Vec<Expr>;

/*#[derive(Debug,PartialEq)]
enum ExprList {
  Cons(Box<Expr>, Box<ExprList>),
  Nil
}*/

#[derive(Debug,PartialEq)]
pub enum Op {
  Plus,
  Minus,
  Multiply,
  Divide
}

fn parse_bin_op(s: &str) -> IResult<&str, Expr> {
  // Parse left expr
  let (s, left_expr) = parse_expr(s)?;

  // Find binary op
  let (s, op) = alt((
    map(char('+'), |_| Op::Plus),
    map(char('-'), |_| Op::Minus),
    map(char('*'), |_| Op::Multiply),
    map(char('/'), |_| Op::Divide),
  ))(s)?;

  // Parse right expr
  let (s, right_expr) = parse_expr(s)?;

  Ok((s, Expr::BinOp(op, Box::new(left_expr), Box::new(right_expr))))
}

fn parse_float(s: &str) -> IResult<&str, Expr> {
  let (s, num) = complete::float(s)?;

  Ok((s, Expr::Float(num)))
}

fn parse_var(s: &str) -> IResult<&str, Expr> {
  let initial_chars: &str = "_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
  let remaining_chars: &str = "_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

  // Returns whole strings matched by the given parser.
  let (s, ident) = recognize(
    // Runs the first parser, if succeeded then runs second, and returns the second result.
    // Note that returned ok value of `preceded()` is ignored by `recognize()`.
    preceded(
      // Parses a single character contained in the given string.
      one_of(initial_chars),
      // Parses the longest slice consisting of the given characters
      is_a(remaining_chars),
    )
  )(s)?;

  Ok((s, Expr::Var(ident.to_string())))
}

pub fn parse_expr(s: &str) -> IResult<&str, Expr> {
  return alt((parse_bin_op, parse_float, parse_var))(s);
}

pub fn parse_program(s: &str) -> IResult<&str, Program> {
  return nom::combinator::all_consuming(separated_list1(tag(";"), parse_expr))(s);
}
