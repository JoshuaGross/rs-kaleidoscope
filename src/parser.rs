extern crate nom;

use crate::ast::{Expr, Op, Program};

use nom::{
  branch::alt,
  bytes::complete::{is_a, tag, take_while, take_until},
  character::complete::{char, one_of, multispace0, multispace1},
  combinator::{map, recognize},
  sequence::{pair, preceded, terminated},
  multi::{fold_many0, fold_many_m_n, many0, separated_list0, separated_list1},
  number::streaming,
  IResult,
};

fn parse_bin_op4(s: &str) -> IResult<&str, Expr> {
  // Parse left/first expr
  let (s, init) = parse_term(s)?;

  // fold expressions
  fold_many0(
    pair(
      preceded(multispace0, terminated(alt((
        map(char('*'), |_| Op::Multiply),
        map(char('/'), |_| Op::Divide)
      )), multispace0)),
      parse_term
    ),
    init,
    |acc, (op, val)| Expr::BinOp(op, Box::new(acc), Box::new(val))
  )(s)
}

fn parse_bin_op3(s: &str) -> IResult<&str, Expr> {
  // Parse left/first expr
  let (s, init) = parse_bin_op4(s)?;

  // fold expressions
  fold_many0(
    pair(
      preceded(multispace0, terminated(alt((
        map(char('+'), |_| Op::Plus),
        map(char('-'), |_| Op::Minus)
      )), multispace0)),
      parse_bin_op4
    ),
    init,
    |acc, (op, val)| Expr::BinOp(op, Box::new(acc), Box::new(val))
  )(s)
}

fn parse_bin_op2(s: &str) -> IResult<&str, Expr> {
  // Parse left/first expr
  let (s, init) = parse_bin_op3(s)?;

  // fold expressions
  fold_many0(
    pair(
      preceded(multispace0, terminated(alt((
        map(char('<'), |_| Op::LessThan),
        map(char('>'), |_| Op::GreaterThan)
      )), multispace0)),
      parse_bin_op3
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
      preceded(multispace0, terminated(alt((
        map(char('|'), |_| Op::BitwiseOr),
        map(char('&'), |_| Op::BitwiseAnd),
      )), multispace0)),
      parse_bin_op2
    ),
    init,
    |acc, (op, val)| Expr::BinOp(op, Box::new(acc), Box::new(val))
  )(s)
}

fn parse_float(s: &str) -> IResult<&str, Expr> {
  let (s, num) = streaming::double(s)?;

  Ok((s, Expr::Float(num)))
}

fn parse_ident(s: &str) -> IResult<&str, String> {
  let initial_chars: &str = "_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
  let remaining_chars: &str = "_abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

  // Returns whole strings matched by the given parser.
  let (s, ident) = recognize(
    // Runs the first parser, if succeeded then runs second, and returns the second result.
    // Note that returned ok value of `preceded()` is ignored by `preceded()`; `recognize` returns
    // the whole string for us.
    preceded(
      // Parses a single character contained in the given string.
      one_of(initial_chars),
      // Parses the longest slice consisting of the given characters
      many0(one_of(remaining_chars)),
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
  let (s, _) = preceded(multispace0, terminated(tag("("), multispace0))(s)?;
  let (s, expr_list) = separated_list0(preceded(multispace0, terminated(tag(","), multispace0)), parse_inner_expr)(s)?;
  let (s, _) = preceded(multispace0, terminated(tag(")"), multispace0))(s)?;
  Ok((s, Expr::Call(ident, expr_list)))
}

fn parse_fn_def(s: &str) -> IResult<&str, Expr> {
  let (s, _) = preceded(multispace0, terminated(tag("def "), multispace0))(s)?;
  let (s, name) = parse_ident(s)?;
  let (s, _) = preceded(multispace0, terminated(is_a("("), multispace0))(s)?;
  let (s, ident_list) = separated_list0(multispace1, parse_ident)(s)?;
  let (s, _) = preceded(multispace0, terminated(is_a(")"), multispace0))(s)?;

  // The body of the function is comprised of a single expression
  let (s, body) = parse_inner_expr(s)?;

  Ok((s, Expr::Function(name, ident_list, Box::new(body))))
}

fn parse_if_stmt(s: &str) -> IResult<&str, Expr> {
  let (s, _) = preceded(multispace0, terminated(tag("if "), multispace0))(s)?;
  let (s, condition) = parse_bin_op1(s)?;
  let (s, _) = preceded(multispace0, terminated(tag("then"), multispace0))(s)?;
  let (s, if_body) = parse_bin_op1(s)?;
  let (s, _) = preceded(multispace0, terminated(tag("else"), multispace0))(s)?;
  let (s, else_body) = parse_inner_expr(s)?;

  Ok((s, Expr::IfExpr(Box::new(condition), Box::new(if_body), Box::new(else_body))))
}

fn parse_for_in_stmt(s: &str) -> IResult<&str, Expr> {
  let (s, _) = preceded(multispace0, terminated(tag("for "), multispace0))(s)?;
  let (s, bound_varname) = parse_ident(s)?;
  let (s, _) = preceded(multispace0, terminated(tag("="), multispace0))(s)?;
  let (s, initial) = parse_inner_expr(s)?;
  let (s, _) = preceded(multispace0, terminated(tag(","), multispace0))(s)?;
  let (s, condition) = parse_inner_expr(s)?;
  let (s, _) = preceded(multispace0, terminated(tag(","), multispace0))(s)?;
  let (s, step) = parse_inner_expr(s)?;
  let (s, _) = preceded(multispace0, terminated(tag("in"), multispace0))(s)?;
  let (s, body) = parse_inner_expr(s)?;

  Ok((s, Expr::ForInExpr(bound_varname, Box::new(initial), Box::new(condition), Box::new(step), Box::new(body))))
}

fn parse_extern_decl(s: &str) -> IResult<&str, Expr> {
  let (s, _) = preceded(multispace0, terminated(tag("extern "), multispace0))(s)?;
  let (s, name) = parse_ident(s)?;
  let (s, _) = preceded(multispace0, terminated(is_a("("), multispace0))(s)?;
  let (s, ident_list) = separated_list0(tag(" "), parse_ident)(s)?;
  let (s, _) = preceded(multispace0, terminated(is_a(")"), multispace0))(s)?;

  Ok((s, Expr::Extern(name, ident_list)))
}

fn parse_term(s: &str) -> IResult<&str, Expr> {
  return alt((parse_call, parse_float, parse_var, parse_parenthetical_term))(s);
}

fn parse_parenthetical_term(s: &str) -> IResult<&str, Expr> {
  let (s, _) = preceded(multispace0, terminated(is_a("("), multispace0))(s)?;
  let (s, res) = parse_inner_expr(s)?;
  let (s, _) = preceded(multispace0, terminated(is_a(")"), multispace0))(s)?;
  Ok((s, res))
}

fn comment(s: &str) -> IResult<&str, ()> {
  let (s, _) = tag("#")(s)?;
  let (s, _) = take_until("\n")(s)?;
  let (s, _) = multispace0(s)?;

  Ok((s, ()))
}

fn multicomment0(s: &str) -> IResult<&str, ()> {
  fold_many0(
    comment,
    (),
    |acc, _| acc
  )(s)
}

fn parse_inner_expr(s: &str) -> IResult<&str, Expr> {
  let (s, initial_expr) = alt((parse_if_stmt, parse_for_in_stmt, parse_bin_op1))(s)?;

  fold_many_m_n(
    0,
    1,
    pair(
      preceded(multispace0, terminated(tag(":"), multispace0)),
      parse_inner_expr
    ),
    initial_expr,
    |acc, (_, expr)| Expr::Sequence(Box::new(acc), Box::new(expr))
  )(s)
}

fn parse_outer_expr(s: &str) -> IResult<&str, Expr> {
  return preceded(multicomment0, terminated(alt((parse_extern_decl, parse_fn_def, parse_inner_expr)), multicomment0))(s);
}

fn parse_program_partial(s: &str) -> IResult<&str, Program> {
  let (s, program) = separated_list1(preceded(multicomment0, terminated(preceded(multispace0, terminated(tag(";"), multispace0)), multicomment0)), parse_outer_expr)(s)?;

  // Consume trailing semicolon if any
  let (s, _) = take_while(|c| c == ';')(s)?;

  // Consume trailing whitespace
  let (s, _) = multispace0(s)?;

  Ok((s, program))
}

pub fn parse_program(s: &str) -> IResult<&str, Program> {
  return nom::combinator::all_consuming(parse_program_partial)(s);
}
