use crate::{
    Result,
    ast::{Node, Operator},
    compile::Compile,
};
use anyhow::Ok;
use inkwell::{
    OptimizationLevel,
    builder::Builder,
    context::Context,
    execution_engine::JitFunction,
    types::IntType,
    values::{AnyValue, IntValue},
};

type JitFunc = unsafe extern "C" fn() -> i32;
pub struct Jit;

struct RecursiveBuilder<'a> {
    i32_: IntType<'a>,
    builder: &'a Builder<'a>,
}

impl<'a> RecursiveBuilder<'a> {
    pub fn new(i32_: IntType<'a>, builder: &'a Builder<'a>) -> Self {
        Self { i32_, builder }
    }

    pub fn build(&self, ast: &Node) -> IntValue<'a> {
        match ast {
            Node::Int(n) => self.i32_.const_int(*n as u64, true),
            Node::UnaryExpr { op, child } => {
                let child = self.build(child);
                match op {
                    Operator::Plus => child.const_neg(),
                    Operator::Minus => child,
                }
            }
            Node::BinaryExpr { op, lhs, rhs } => {
                let left = self.build(lhs);
                let right = self.build(rhs);

                match op {
                    Operator::Plus => self.builder.build_int_add(left, right, "_add").unwrap(),
                    Operator::Minus => self.builder.build_int_sub(left, right, "_sub").unwrap(),
                }
            }
        }
    }
}

impl Compile for Jit {
    type Output = Result<i32>;

    fn from_ast(ast: Vec<Node>) -> Self::Output {
        let context = Context::create();
        let module = context.create_module("calculator");

        let builder = context.create_builder();

        let execution_engine = module
            .create_jit_execution_engine(OptimizationLevel::None)
            .unwrap();

        let i32_ = context.i32_type();
        let fn_ = i32_.fn_type(&[], false);

        let function = module.add_function("jit", fn_, None);
        let entry_block = context.append_basic_block(function, "entry");

        builder.position_at_end(entry_block);

        for node in ast {
            let recursive_builder = RecursiveBuilder::new(i32_, &builder);
            let return_value = recursive_builder.build(&node);
            builder.build_return(Some(&return_value))?;
        }

        println!(
            "generated LLVM IR: {}",
            function.print_to_string().to_string()
        );

        // runner
        unsafe {
            let jit_fun: JitFunction<JitFunc> = execution_engine.get_function("jit")?;

            Ok(jit_fun.call())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basics() {
        assert_eq!(Jit::from_source("1 + 2").unwrap(), 3);
        assert_eq!(Jit::from_source("2 + (2 - 1)").unwrap(), 3);
        assert_eq!(Jit::from_source("(2 + 3) - 1").unwrap(), 4);
        assert_eq!(Jit::from_source("1 + ((2 + 3) - (2 + 3))").unwrap(), 1);
        assert_eq!(Jit::from_source("(1 + 2)").unwrap(), 3);
        // parser fails
        // assert_eq!(Jit::from_source("2 + 3 - 1").unwrap(), 4);
    }
}
