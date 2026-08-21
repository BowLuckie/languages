use crate::{interpreter::Interpreter, parser::parse};

pub mod ast;
pub mod interpreter;
pub mod parser;
pub mod preprocessor;

pub fn run(source: &str) -> Result<(), String> {
    let program = parse(source)?;
    let mut interpreter = Interpreter::new();
    interpreter.run(&program)
}
