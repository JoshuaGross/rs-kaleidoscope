mod parser;

use std::fs::File;
use std::io::prelude::*;

fn main() -> std::io::Result<()> {
  let filename = std::env::args().nth(1).expect("no filename given");

  let mut file = File::open(filename)?;
  let mut contents = String::new();
  file.read_to_string(&mut contents)?;

  let res = parser::parse_program(&contents).unwrap();
  println!("Parsed: {:?}", res.1);
  Ok(())
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
  assert_eq!(parser::parse_program("five 1"), Err(nom::Err::Error(nom::error::Error::new(&" 1"[..], nom::error::ErrorKind::Eof))));

  // Parse basic arithmetic
  assert_eq!(parser::parse_program("five+1.4"), Ok(("", vec![Expr::BinOp(Op::Plus, Box::new(Expr::Var("five".to_string())), Box::new(Expr::Float(1.4)))])));
  assert_eq!(parser::parse_program("five + 1.4"), Ok(("", vec![Expr::BinOp(Op::Plus, Box::new(Expr::Var("five".to_string())), Box::new(Expr::Float(1.4)))])));
  assert_eq!(parser::parse_program("6 * 7"), Ok(("", vec![Expr::BinOp(Op::Multiply, Box::new(Expr::Float(6.0)), Box::new(Expr::Float(7.0)))])));

  // Parse arithmetic with precedence
  assert_eq!(parser::parse_program("5 + 6 * 7"), Ok(("", vec![Expr::BinOp(Op::Plus, Box::new(Expr::Float(5.0)), Box::new(Expr::BinOp(Op::Multiply, Box::new(Expr::Float(6.0)), Box::new(Expr::Float(7.0)))))])));

  // Parse arithmetic with parenthetical
  assert_eq!(parser::parse_program("5 * (6 + 7)"), Ok(("", vec![Expr::BinOp(Op::Multiply, Box::new(Expr::Float(5.0)), Box::new(Expr::BinOp(Op::Plus, Box::new(Expr::Float(6.0)), Box::new(Expr::Float(7.0)))))])));

  // Parse call
  assert_eq!(parser::parse_program("foobar()"), Ok(("", vec![Expr::Call("foobar".to_string(), vec![])])));
  assert_eq!(parser::parse_program("foobar(1, 2)"), Ok(("", vec![Expr::Call("foobar".to_string(), vec![Expr::Float(1.0), Expr::Float(2.0)])])));
  assert_eq!(parser::parse_program("foobar(1, 2, 3+4)"), Ok(("", vec![Expr::Call("foobar".to_string(), vec![Expr::Float(1.0), Expr::Float(2.0), Expr::BinOp(Op::Plus, Box::new(Expr::Float(3.0)), Box::new(Expr::Float(4.0)))])])));
  assert_eq!(parser::parse_program("foobar(1, 2, 3+4, baz() )"), Ok(("", vec![Expr::Call("foobar".to_string(), vec![Expr::Float(1.0), Expr::Float(2.0), Expr::BinOp(Op::Plus, Box::new(Expr::Float(3.0)), Box::new(Expr::Float(4.0))), Expr::Call("baz".to_string(), vec![])])])));

  // Parse function definitions
  assert_eq!(parser::parse_program("def foobar(term1 term2 term3) { baz(term1 + term2 + term3) }"), Ok(("",
    vec![
      Expr::Function("foobar".to_string(),
        vec!["term1".to_string(), "term2".to_string(), "term3".to_string()],
        vec![Expr::Call(
          "baz".to_string(),
          vec![
            Expr::BinOp(Op::Plus,
                        Box::new(Expr::BinOp(Op::Plus,
                                             Box::new(Expr::Var("term1".to_string())),
                                             Box::new(Expr::Var("term2".to_string()))
                                             )),
                        Box::new(Expr::Var("term3".to_string()))
                       )
          ]
        )
      ])
    ]
  )));

  // extern
  assert_eq!(parser::parse_program("extern foobar(param1 param2 param3)"), Ok(("", vec![Expr::Extern("foobar".to_string(), vec!["param1".to_string(), "param2".to_string(), "param3".to_string()])])));

  // This looks correct
  //assert_eq!(parser::parse_program("extern foobar(param1 param2 param3); def foo(item1) { foobar(item1 + 2); baz(17) }"), Ok(("", vec![Expr::Extern("foobar".to_string(), vec!["param1".to_string(), "param2".to_string(), "param3".to_string()])])));
}

