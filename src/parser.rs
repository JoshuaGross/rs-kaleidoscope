extern crate nom;

use nom::{
  branch::alt,
  bytes::complete::{is_a, tag},
  character::complete::{char, one_of, multispace0},
  combinator::{map, recognize},
  sequence::{delimited, pair, preceded},
  multi::{fold_many0, separated_list0, separated_list1},
  number::complete,
  IResult,
};

type Name = String;

#[derive(Clone, Debug,PartialEq)]
pub enum Expr {
  Float(f32),
  Var(Name),
  BinOp(Op, Box<Expr>, Box<Expr>),
  Call(Name, Program),
  Function(Name, Vec<Name>, Program),
  Extern(Name, Vec<Name>)
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
      delimited(multispace0, alt((
        map(char('*'), |_| Op::Multiply),
        map(char('/'), |_| Op::Divide)
      )), multispace0),
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
      delimited(multispace0, alt((
        map(char('+'), |_| Op::Plus),
        map(char('-'), |_| Op::Minus)
      )), multispace0),
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

fn parse_ident(s: &str) -> IResult<&str, String> {
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

  Ok((s, ident.to_string()))
}

fn parse_var(s: &str) -> IResult<&str, Expr> {
  let (s, ident) = parse_ident(s)?;
  Ok((s, Expr::Var(ident)))
}

fn parse_call(s: &str) -> IResult<&str, Expr> {
  let (s, ident) = parse_ident(s)?;
  let (s, _) = delimited(multispace0, is_a("("), multispace0)(s)?;
  let (s, expr_list) = separated_list0(delimited(multispace0, tag(","), multispace0), parse_expr)(s)?;
  let (s, _) = delimited(multispace0, is_a(")"), multispace0)(s)?;
  Ok((s, Expr::Call(ident, expr_list)))
}

fn parse_fn_def(s: &str) -> IResult<&str, Expr> {
  let (s, _) = delimited(multispace0, tag("def "), multispace0)(s)?;
  let (s, name) = parse_ident(s)?;
  let (s, _) = delimited(multispace0, is_a("("), multispace0)(s)?;
  let (s, ident_list) = separated_list0(tag(" "), parse_ident)(s)?;
  let (s, _) = delimited(multispace0, is_a(")"), multispace0)(s)?;
  let (s, _) = delimited(multispace0, is_a("{"), multispace0)(s)?;
  let (s, expr_list) = parse_program_partial(s)?;
  let (s, _) = delimited(multispace0, is_a("}"), multispace0)(s)?;

  Ok((s, Expr::Function(name, ident_list, expr_list)))
}

fn parse_extern_decl(s: &str) -> IResult<&str, Expr> {
  let (s, _) = delimited(multispace0, tag("extern "), multispace0)(s)?;
  let (s, name) = parse_ident(s)?;
  let (s, _) = delimited(multispace0, is_a("("), multispace0)(s)?;
  let (s, ident_list) = separated_list0(tag(" "), parse_ident)(s)?;
  let (s, _) = delimited(multispace0, is_a(")"), multispace0)(s)?;

  Ok((s, Expr::Extern(name, ident_list)))
}

fn parse_term(s: &str) -> IResult<&str, Expr> {
  return alt((parse_call, parse_float, parse_var, parse_parenthetical_term))(s);
}

fn parse_parenthetical_term(s: &str) -> IResult<&str, Expr> {
  let (s, _) = is_a("(")(s)?;
  let (s, res) = parse_expr(s)?;
  let (s, _) = is_a(")")(s)?;
  Ok((s, res))
}

pub fn parse_expr(s: &str) -> IResult<&str, Expr> {
  return alt((parse_extern_decl, parse_fn_def, parse_bin_op1))(s);
}

fn parse_program_partial(s: &str) -> IResult<&str, Program> {
  return separated_list1(delimited(multispace0, tag(";"), multispace0), parse_expr)(s);
}

pub fn parse_program(s: &str) -> IResult<&str, Program> {
  return nom::combinator::all_consuming(parse_program_partial)(s);
}
