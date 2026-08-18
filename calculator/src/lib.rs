pub mod ast;
pub mod compile;
pub mod parser;
pub mod typing;

pub type Result<T> = anyhow::Result<T>;
