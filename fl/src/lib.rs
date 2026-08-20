use crate::{
    interpreter::{Interpreter, Value},
    parser::parse,
};

pub mod ast;
pub mod interpreter;
pub mod parser;

pub fn run(source: &str) -> Result<Value, String> {
    let program = parse(source)?;
    let mut interpreter = Interpreter::new();
    interpreter.run(&program)
}
