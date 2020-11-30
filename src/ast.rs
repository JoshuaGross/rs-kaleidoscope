pub type Name = String;

#[derive(Clone, Debug,PartialEq)]
pub enum Expr {
  Float(f64),
  Var(Name),
  BinOp(Op, Box<Expr>, Box<Expr>),
  Call(Name, Program),
  Function(Name, Vec<Name>, Box<Expr>),
  IfExpr(Box<Expr>, Box<Expr>, Box<Expr>),
  ForInExpr(Name, Box<Expr>, Box<Expr>, Box<Expr>, Box<Expr>),
  Extern(Name, Vec<Name>)
}

pub type Program = Vec<Expr>;

#[derive(Clone, Debug,PartialEq)]
pub enum Op {
  Plus,
  Minus,
  Multiply,
  Divide,
  LessThan,
  GreaterThan
}
