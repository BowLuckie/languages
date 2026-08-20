use core::fmt;
use std::collections::HashMap;

use crate::ast::*;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Bool(bool),
    Function {
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    Unit, // ()
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Function { params, .. } => write!(f, "<function({})>", params.join(", ")),
            Value::Unit => write!(f, "()"),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
struct Frame {
    locals: HashMap<String, Value>,
}

impl Frame {
    fn new() -> Self {
        Self::default()
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct Interpreter {
    globals: HashMap<String, Value>,
    call_stack: Vec<Frame>,
}

enum ControlFlow {
    Continue(Value),
    Return(Value),
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            call_stack: vec![Frame::new()],
            ..Default::default()
        }
    }

    pub fn run(&mut self, program: &Program) -> Result<Value, String> {
        let result = Value::Unit;
        for stmt in program {}
        Ok(result)
    }
}
