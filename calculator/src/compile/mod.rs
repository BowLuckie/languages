use crate::{ast::Node, parser};

pub mod interpreter;
pub mod jit;
pub mod vm;

pub trait Compile {
    type Output;

    fn from_ast(ast: Vec<Node>) -> Self::Output;

    fn from_source(source: &str) -> Self::Output {
        println!("Compiling...");
        let ast: Vec<Node> = parser::parse(source)
            .unwrap_or_else(|err| panic!("a parser error has occurred! {}", err));
        println!("AST - {:?}", ast);
        Self::from_ast(ast)
    }
}
