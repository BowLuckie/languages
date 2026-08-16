#![allow(clippy::module_inception)]

mod bytecode;
mod opcode;
mod vmcore;

pub use vmcore::VM;
