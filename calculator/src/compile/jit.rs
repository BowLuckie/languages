use crate::{
    ast::{Node, Operator},
    compile::Compile,
    typing::{Ty, infer_type},
};
use inkwell::{
    OptimizationLevel,
    builder::Builder,
    context::Context,
    execution_engine::JitFunction,
    types::{FloatType, IntType},
    values::{BasicValueEnum, FloatValue, IntValue},
};

pub struct Jit;

struct RecursiveBuilder<'a> {
    i32_: IntType<'a>,
    f32_: FloatType<'a>,
    builder: &'a Builder<'a>,
}

#[allow(non_camel_case_types)]
enum NumVal<'a> {
    i32_(IntValue<'a>),
    f32_(FloatValue<'a>),
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum RetVal {
    i32_(i32),
    f32_(f32),
}

impl<'a> TryFrom<NumVal<'a>> for IntValue<'a> {
    type Error = &'static str;
    fn try_from(val: NumVal<'a>) -> Result<Self, Self::Error> {
        match val {
            NumVal::i32_(int_val) => Ok(int_val),
            NumVal::f32_(_) => Err("expected int found float"),
        }
    }
}

impl<'a> TryFrom<NumVal<'a>> for FloatValue<'a> {
    type Error = &'static str;
    fn try_from(val: NumVal<'a>) -> Result<Self, Self::Error> {
        match val {
            NumVal::f32_(float_val) => Ok(float_val),
            NumVal::i32_(_) => Err("expected float found int"),
        }
    }
}

impl<'a> From<IntValue<'a>> for NumVal<'a> {
    fn from(int_val: IntValue<'a>) -> Self {
        NumVal::i32_(int_val)
    }
}

impl<'a> From<FloatValue<'a>> for NumVal<'a> {
    fn from(float_val: FloatValue<'a>) -> Self {
        NumVal::f32_(float_val)
    }
}

impl<'a> From<NumVal<'a>> for BasicValueEnum<'a> {
    fn from(val: NumVal<'a>) -> Self {
        match val {
            NumVal::i32_(int_val) => int_val.into(),
            NumVal::f32_(float_val) => float_val.into(),
        }
    }
}

impl<'a> RecursiveBuilder<'a> {
    pub fn new(i32_: IntType<'a>, f32_: FloatType<'a>, builder: &'a Builder<'a>) -> Self {
        Self {
            i32_,
            builder,
            f32_,
        }
    }

    pub fn build(&self, ast: &Node) -> NumVal<'a> {
        match ast {
            Node::Int(n) => self.i32_.const_int(*n as u64, true).into(),
            Node::Float(f) => self.f32_.const_float(*f as f64).into(),
            Node::UnaryExpr { op, child } => {
                let child = self.build(child);
                match child {
                    NumVal::i32_(int_value) => match op {
                        Operator::Min => int_value.const_neg().into(),
                        Operator::Plus => int_value.into(),
                        _ => panic!("cannot apply unary with a scalar operator"),
                    },
                    NumVal::f32_(float_value) => match op {
                        Operator::Plus => float_value.into(),
                        Operator::Min => self
                            .builder
                            .build_float_neg(float_value, "negtmp")
                            .unwrap()
                            .into(),
                        _ => panic!("cannot apply unary with a scalar operator"),
                    },
                }
            }
            Node::BinaryExpr { op, lhs, rhs } => {
                let left = self.build(lhs);
                let right = self.build(rhs);
                match (left, right) {
                    (NumVal::i32_(il), NumVal::i32_(ir)) => self.int_operation(il, ir, *op).into(),
                    (NumVal::i32_(il), NumVal::f32_(fr)) => {
                        self.float_operation(self.create_float(il), fr, *op).into()
                    }

                    (NumVal::f32_(fl), NumVal::i32_(ir)) => {
                        self.float_operation(fl, self.create_float(ir), *op).into()
                    }
                    (NumVal::f32_(fl), NumVal::f32_(fr)) => {
                        self.float_operation(fl, fr, *op).into()
                    }
                }
            }
        }
    }

    fn int_operation(&self, left: IntValue<'a>, right: IntValue<'a>, op: Operator) -> IntValue<'a> {
        match op {
            Operator::Plus => self.builder.build_int_add(left, right, "_add").unwrap(),
            Operator::Min => self.builder.build_int_sub(left, right, "_sub").unwrap(),
            Operator::Mul => self.builder.build_int_mul(left, right, "_mul").unwrap(),
            Operator::Div => self
                .builder
                .build_int_signed_div(left, right, "_div")
                .unwrap(),
        }
    }

    fn create_float(&self, int: IntValue<'a>) -> FloatValue<'a> {
        self.builder
            .build_signed_int_to_float(int, self.f32_, "_int_to_float")
            .unwrap()
    }

    fn float_operation(
        &self,
        left: FloatValue<'a>,
        right: FloatValue<'a>,
        op: Operator,
    ) -> FloatValue<'a> {
        match op {
            Operator::Plus => self.builder.build_float_add(left, right, "_fadd").unwrap(),
            Operator::Min => self.builder.build_float_sub(left, right, "_fsub").unwrap(),
            Operator::Mul => self.builder.build_float_mul(left, right, "_fmul").unwrap(),
            Operator::Div => self.builder.build_float_div(left, right, "_fdiv").unwrap(),
        }
    }
}

impl Compile for Jit {
    type Output = crate::Result<RetVal>;

    fn from_ast(ast: Vec<Node>) -> Self::Output {
        let context = Context::create();
        let module = context.create_module("calculator");

        let builder = context.create_builder();

        let execution_engine = module
            .create_jit_execution_engine(OptimizationLevel::None)
            .unwrap();

        let i32_ = context.i32_type();
        let f32_ = context.f32_type();

        let result_ty = infer_type(&ast);

        let (fn_name, _function, entry_block) = match result_ty {
            Ty::Int => {
                let fn_type = i32_.fn_type(&[], false);
                let fun = module.add_function("jit", fn_type, None);
                let block = context.append_basic_block(fun, "entry");
                ("jit", fun, block)
            }
            Ty::Float => {
                let fn_type = f32_.fn_type(&[], false);
                let fun = module.add_function("jit", fn_type, None);
                let block = context.append_basic_block(fun, "entry");
                ("jit", fun, block)
            }
        };

        builder.position_at_end(entry_block);
        let recursive_builder = RecursiveBuilder::new(i32_, f32_, &builder);
        let mut last = None;
        for node in &ast {
            last = Some(recursive_builder.build(node));
        }

        match last.unwrap() {
            NumVal::i32_(i) => {
                builder.build_return(Some(&i))?;
            }
            NumVal::f32_(f) => {
                builder.build_return(Some(&f))?;
            }
        }

        // runner
        type JitFuncI = unsafe extern "C" fn() -> i32;
        type JitFuncF = unsafe extern "C" fn() -> f32;

        unsafe {
            match result_ty {
                Ty::Int => {
                    let jit_fun: JitFunction<JitFuncI> = execution_engine.get_function(fn_name)?;
                    Ok(RetVal::i32_(jit_fun.call()))
                }
                Ty::Float => {
                    let jit_fun: JitFunction<JitFuncF> = execution_engine.get_function(fn_name)?;
                    Ok(RetVal::f32_(jit_fun.call()))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn int() {
        assert_eq!(Jit::from_source("1 + 2").unwrap(), RetVal::i32_(3));
        assert_eq!(Jit::from_source("2 + (2 - 1)").unwrap(), RetVal::i32_(3));
        assert_eq!(Jit::from_source("(2 + 3) - 1").unwrap(), RetVal::i32_(4));
        assert_eq!(Jit::from_source("(1 + 2)").unwrap(), RetVal::i32_(3));
        assert_eq!(Jit::from_source("2 + 3 - 1").unwrap(), RetVal::i32_(4));
        assert_eq!(
            Jit::from_source("1 + ((2 + 3) - (2 + 3))").unwrap(),
            RetVal::i32_(1)
        );
    }

    #[test]
    fn float() {
        // Basic Float Literals
        assert_eq!(Jit::from_source("42.0").unwrap(), RetVal::f32_(42.0));
        assert_eq!(Jit::from_source("0.5").unwrap(), RetVal::f32_(0.5));

        // Basic Left-to-Right Operations
        assert_eq!(Jit::from_source("3.1 + 1.0").unwrap(), RetVal::f32_(4.1));
        assert_eq!(Jit::from_source("5.5 - 2.1").unwrap(), RetVal::f32_(3.4));
        assert_eq!(Jit::from_source("1.5 * 2.0").unwrap(), RetVal::f32_(3.0));
        assert_eq!(Jit::from_source("7.5 / 2.5").unwrap(), RetVal::f32_(3.0));

        // Unary Minus with Floats
        assert_eq!(Jit::from_source("-2.5").unwrap(), RetVal::f32_(-2.5));
        assert_eq!(Jit::from_source("-0.0").unwrap(), RetVal::f32_(-0.0));

        assert_eq!(
            Jit::from_source("1.0 + 2.0 * 3.0").unwrap(),
            RetVal::f32_(9.0)
        );

        // Strict Left-to-Right chaining
        // ((10.0 / 2.0) - 1.0) * 0.5 = (5.0 - 1.0) * 0.5 = 4.0 * 0.5 = 2.0
        assert_eq!(
            Jit::from_source("10.0 / 2.0 - 1.0 * 0.5").unwrap(),
            RetVal::f32_(2.0)
        );

        // Complex chain with unary operation at the start
        // ((-5.0 * 2.0) + 12.5) = -10.0 + 12.5 = 2.5
        assert_eq!(
            Jit::from_source("-5.0 * 2.0 + 12.5").unwrap(),
            RetVal::f32_(2.5)
        );
    }
}
