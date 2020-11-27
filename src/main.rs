mod parser;

use std::fs::File;
use std::io::prelude::*;
use std::error::Error;

use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::passes::PassManager;
use inkwell::values::{FunctionValue};
use inkwell::{OptimizationLevel};
use inkwell::execution_engine::{ExecutionEngine, JitFunction};

/// Convenience type alias for the `sum` function.
///
/// Calling this is innately `unsafe` because there's no guarantee it doesn't
/// do `unsafe` operations internally.
type SumFunc = unsafe extern "C" fn(u64, u64, u64) -> u64;

pub struct CodeGen<'a, 'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub fpm: &'a PassManager<FunctionValue<'ctx>>,
    pub execution_engine: ExecutionEngine<'ctx>,
    // pub function: &'a Function,
}

impl<'a, 'ctx> CodeGen<'a, 'ctx> {
    fn jit_compile_sum(&self) -> Option<JitFunction<SumFunc>> {
        let i64_type = self.context.i64_type();
        let fn_type = i64_type.fn_type(&[i64_type.into(), i64_type.into(), i64_type.into()], false);
        let function = self.module.add_function("sum", fn_type, None);
        let basic_block = self.context.append_basic_block(function, "entry");

        self.builder.position_at_end(basic_block);

        let x = function.get_nth_param(0)?.into_int_value();
        let y = function.get_nth_param(1)?.into_int_value();
        let z = function.get_nth_param(2)?.into_int_value();

        let sum = self.builder.build_int_add(x, y, "sum");
        let sum = self.builder.build_int_add(sum, z, "sum");

        self.builder.build_return(Some(&sum));

        // Verify the function
        if function.verify(true) {
          // Print function before optimization
          println!("Function before optimization:");
          function.print_to_stderr();

          // Optimize the function
          self.fpm.run_on(&function);

          // Print function after optimization
          println!("Function after optimization:");
          function.print_to_stderr();

          // JIT compile the function
          unsafe { self.execution_engine.get_function("sum").ok() }
        } else {
          unsafe { function.delete(); }

          None
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
  let filename = std::env::args().nth(1).expect("no filename given");

  let mut file = File::open(filename)?;
  let mut contents = String::new();
  file.read_to_string(&mut contents)?;

  let res = parser::parse_program(&contents).unwrap();
  println!("Parsed: {:?}", res.1);

  // Create codegen
  let context = Context::create();
  let module = context.create_module("sum"); // could be repl, tmp, etc
  let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None)?;
  let fpm = PassManager::create(&module);

  fpm.add_instruction_combining_pass();
  fpm.add_reassociate_pass();
  fpm.add_gvn_pass();
  fpm.add_cfg_simplification_pass();
  fpm.add_basic_alias_analysis_pass();
  fpm.add_promote_memory_to_register_pass();
  fpm.add_instruction_combining_pass();
  fpm.add_reassociate_pass();

  fpm.initialize();

  let codegen = CodeGen {
      context: &context,
      module,
      builder: context.create_builder(),
      fpm: &fpm,
      execution_engine,
  };

  let sum_fn = codegen.jit_compile_sum().ok_or("Unable to JIT compile `sum`")?;

  let x = 1u64;
  let y = 2u64;
  let z = 3u64;
  let res = unsafe { sum_fn.call(x, y, z) };
  println!("{} + {} + {} = {}", x, y, z, res);
  assert_eq!(res, x + y + z);

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

