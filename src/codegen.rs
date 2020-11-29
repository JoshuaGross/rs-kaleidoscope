use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::passes::PassManager;
use inkwell::types::BasicTypeEnum;
use inkwell::values::{BasicValue, BasicValueEnum, FloatValue, FunctionValue, PointerValue};
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::{OptimizationLevel, FloatPredicate};

use std::collections::HashMap;
use std::error::Error;

use crate::ast::{Expr, Name, Op, Program};

use std::io::Write;

/**
 * Library functions.
 */

// macro used to print & flush without printing a new line
macro_rules! print_flush {
    ( $( $x:expr ),* ) => {
        print!( $($x, )* );

        std::io::stdout().flush().expect("Could not flush to standard output.");
    };
}

#[no_mangle]
pub extern fn putchard(x: f64) -> f64 {
    print_flush!("{}", x as u8 as char);
    x
}

#[no_mangle]
pub extern fn printd(x: f64) -> f64 {
    println!("{}", x);
    x
}

// Adding the functions above to a global array,
// so Rust compiler won't remove them.
#[used]
static EXTERNAL_FNS: [extern fn(f64) -> f64; 2] = [putchard, printd];

/// Convenience type alias for functions.
///
/// Calling this is innately `unsafe` because there's no guarantee it doesn't
/// do `unsafe` operations internally.
//type SumFunc = unsafe extern "C" fn(u64, u64, u64) -> u64;
type MainFunc = unsafe extern "C" fn() -> f64;

pub struct CodeGen<'a, 'ctx> {
    pub context: &'ctx Context,
    pub module: Module<'ctx>,
    pub builder: Builder<'ctx>,
    pub fpm: &'a PassManager<FunctionValue<'ctx>>,
    pub execution_engine: ExecutionEngine<'ctx>,

    variables: HashMap<String, PointerValue<'ctx>>,
    fn_value_opt: Option<FunctionValue<'ctx>>
}
impl<'a, 'ctx> CodeGen<'a, 'ctx> {
    /*pub fn jit_compile_sum(&self) -> Option<JitFunction<SumFunc>> {
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
    }*/

    // Gets a defined function given its name.
    // Directly from https://github.com/TheDan64/inkwell/blob/master/examples/kaleidoscope/main.rs
    #[inline]
    fn get_function(&self, name: &str) -> Option<FunctionValue<'ctx>> {
        self.module.get_function(name)
    }

    // Returns the `FunctionValue` representing the function currently being compiled.
    // Directly from https://github.com/TheDan64/inkwell/blob/master/examples/kaleidoscope/main.rs
    #[inline]
    fn fn_value(&self) -> FunctionValue<'ctx> {
        self.fn_value_opt.unwrap()
    }

    // Creates a new stack allocation instruction in the entry block of the function.
    // Directly from https://github.com/TheDan64/inkwell/blob/master/examples/kaleidoscope/main.rs
    fn create_entry_block_alloca(&self, name: &str) -> PointerValue<'ctx> {
        let builder = self.context.create_builder();

        let entry = self.fn_value().get_first_basic_block().unwrap();

        match entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(entry)
        }

        builder.build_alloca(self.context.f64_type(), name)
    }

    fn compile_expr(&mut self, expr: &Expr) -> Result<FloatValue<'ctx>, &'static str> {
        match &*expr {
            Expr::Float(nb) => Ok(self.context.f64_type().const_float(*nb)),

            Expr::Var(ref name) => {
                match self.variables.get(name.as_str()) {
                    Some(var) => Ok(self.builder.build_load(*var, name.as_str()).into_float_value()),
                    None => Err("Could not find a matching variable.")
                }
            },

            Expr::BinOp(ref op, ref left, ref right) => {
                let lhs = self.compile_expr(left)?;
                let rhs = self.compile_expr(right)?;

                match &*op {
                    Op::Plus        => Ok(self.builder.build_float_add(lhs, rhs, "tmpadd")),
                    Op::Minus       => Ok(self.builder.build_float_add(lhs, rhs, "tmpsub")),
                    Op::Divide      => Ok(self.builder.build_float_add(lhs, rhs, "tmpdiv")),
                    Op::Multiply    => Ok(self.builder.build_float_add(lhs, rhs, "tmpmul")),
                    Op::LessThan    => Ok({
                        let cmp = self.builder.build_float_compare(FloatPredicate::ULT, lhs, rhs, "tmpcmp");
                        self.builder.build_unsigned_int_to_float(cmp, self.context.f64_type(), "tmpbool")
                    }),
                    Op::GreaterThan => Ok({
                        let cmp = self.builder.build_float_compare(FloatPredicate::ULT, rhs, lhs, "tmpcmp");
                        self.builder.build_unsigned_int_to_float(cmp, self.context.f64_type(), "tmpbool")
                    }),
                    _ => Err("Unsupported binary operation")
                }
            },

            Expr::Call(ref fn_name, ref args) => {
                match self.get_function(fn_name.as_str()) {
                    Some(fun) => {
                        let mut compiled_args = Vec::with_capacity(args.len());

                        for arg in args {
                            compiled_args.push(self.compile_expr(arg)?);
                        }

                        let argsv: Vec<BasicValueEnum> = compiled_args.iter().by_ref().map(|&val| val.into()).collect();

                        match self.builder.build_call(fun, argsv.as_slice(), "tmp").try_as_basic_value().left() {
                            Some(value) => Ok(value.into_float_value()),
                            None => Err("Invalid call produced.")
                        }
                    },
                    None => Err("Unknown function.")
                }
            },

            x => {
                println!("This type of expr not yet supported: {:?}", x);
                Err("Expr not yet supported")
            }
        }
    }

    fn compile_prototype(&self, name: &str, params: &Vec<Name>) -> Result<FunctionValue<'ctx>, String> {
        // All functions return f64
        let ret_type = self.context.f64_type();

        // All arguments are of type f64
        let args_types = std::iter::repeat(ret_type)
            .take(params.len())
            .map(|f| f.into())
            .collect::<Vec<BasicTypeEnum>>();
        let args_types = args_types.as_slice();

        let fn_type = self.context.f64_type().fn_type(args_types, false);
        let fn_val = self.module.add_function(name, fn_type, None);

        // set arguments names
        for (i, arg) in fn_val.get_param_iter().enumerate() {
            arg.into_float_value().set_name(params[i].as_str());
        }

        // finally return built prototype
        Ok(fn_val)
    }

    // Compiles the specified `Function` into an LLVM `FunctionValue`.
    fn compile_fn(&mut self, name: &str, params: &Vec<Name>, expr: &Box<Expr>) -> Result<FunctionValue, String> {
        let function = self.compile_prototype(&name, &params)?;
        let entry = self.context.append_basic_block(function, "entry");

        self.builder.position_at_end(entry);

        // update fn field
        self.fn_value_opt = Some(function);

        // build variables map
        self.variables.reserve(params.len());

        for (i, arg) in function.get_param_iter().enumerate() {
            let arg_name = params[i].as_str();
            let alloca = self.create_entry_block_alloca(arg_name);

            self.builder.build_store(alloca, arg);

            self.variables.insert(params[i].clone(), alloca);
        }

        // compile body
        let body = self.compile_expr(expr.as_ref())?;

        self.builder.build_return(Some(&body));

        // return the whole thing after verification and optimization
        if function.verify(true) {
            self.fpm.run_on(&function);

            Ok(function)
        } else {
            unsafe {
                function.delete();
            }

            Err("Invalid generated function.".to_string())
        }
    }


    pub fn jit_compile_main(&self) -> Option<JitFunction<MainFunc>> {
      self.module.print_to_stderr();

      unsafe { self.execution_engine.get_function("main").ok() }
    }

    pub fn compile_program(&mut self, exprs: &Program) -> Result<(), String> {
        for expr in exprs {
            match expr {
                Expr::Function(name, params, expr) => {
                    self.compile_fn(&name, &params, &expr)?;
                },
                Expr::Extern(name, params) => {
                    self.compile_prototype(&name, &params)?;
                },
                _ => {
                    return Err("Only functions and `extern` declarations can be at the outer level".to_string());
                }
            }
        }

        Ok(())
    }

    pub fn mk_compiler(
        context: &'ctx Context,
        pass_manager: &'a PassManager<FunctionValue<'ctx>>,
        module: Box<Module<'ctx>>
    ) -> Result<CodeGen<'a, 'ctx>, Box<dyn Error>> {
      let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None)?;
      Ok(CodeGen {
          context: &context,
          module: *module,
          builder: context.create_builder(),
          fpm: &pass_manager,
          execution_engine: execution_engine,
          fn_value_opt: None,
          variables: HashMap::new()
      })
    }
}
