use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::passes::PassManager;
use inkwell::values::{FunctionValue};
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
    pub fn jit_compile_sum(&self) -> Option<JitFunction<SumFunc>> {
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
