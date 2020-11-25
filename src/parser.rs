extern crate nom;

use nom::{
  branch::alt,
  bytes::complete::{is_a, tag},
  character::complete::{char, one_of, space0},
  combinator::{map, recognize},
  sequence::{delimited, pair, preceded},
  multi::{fold_many0, separated_list1},
  number::complete,
  IResult
};

type Name = String;

#[derive(Clone, Debug,PartialEq)]
pub enum Expr {
  Float(f32),
  Var(Name),
  BinOp(Op, Box<Expr>, Box<Expr>),
  /*Call(Name, ExprList),
  Function(Name, ExprList, Box<Expr>),
  Extern(Name, ExprList)*/
}

pub type Program = Vec<Expr>;

#[derive(Clone, Debug,PartialEq)]
pub enum Op {
  Plus,
  Minus,
  Multiply,
  Divide
}

fn parse_bin_op2(s: &str) -> IResult<&str, Expr> {
  // Parse left/first expr
  let (s, init) = parse_term(s)?;

  // fold expressions
  fold_many0(
    pair(
      alt((
        map(char('*'), |_| Op::Multiply),
        map(char('/'), |_| Op::Divide)
      )),
      parse_term
    ),
    init,
    |acc, (op, val)| Expr::BinOp(op, Box::new(acc), Box::new(val))
  )(s)
}

fn parse_bin_op1(s: &str) -> IResult<&str, Expr> {
  // Parse left/first expr
  let (s, init) = parse_bin_op2(s)?;

  // fold expressions
  fold_many0(
    pair(
      alt((
        map(char('+'), |_| Op::Plus),
        map(char('-'), |_| Op::Minus)
      )),
      parse_bin_op2
    ),
    init,
    |acc, (op, val)| Expr::BinOp(op, Box::new(acc), Box::new(val))
  )(s)
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

pub fn parse_term(s: &str) -> IResult<&str, Expr> {
  return delimited(space0, alt((parse_float, parse_var)), space0)(s);
}

pub fn parse_expr(s: &str) -> IResult<&str, Expr> {
  return delimited(space0, alt((parse_bin_op1, parse_term)), space0)(s);
}

pub fn parse_program(s: &str) -> IResult<&str, Program> {
  return nom::combinator::all_consuming(separated_list1(tag(";"), parse_expr))(s);
}
