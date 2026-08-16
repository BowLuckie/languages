use crate::{
    ast::Node,
    compile::vm::{bytecode::Bytecode, opcode::Byte},
};

const STACK_SIZE: usize = usize::pow(2, 9); // 2^9

pub struct VM {
    bytecode: Bytecode,
    stack: [Node; STACK_SIZE],
    top: usize,
}

impl VM {
    pub fn run(&self) {
        let mut ip = 0;
        while ip < self.bytecode.instructions.len() {
            let instruction_addr = ip;
            ip += 1;

            let op: Byte = self.bytecode.instructions[ip]; // hex of bytecode

            match op {
                0x01 => { // OpConst
                }
                _ => panic!("unknown instruction! {}", op),
            }
        }
    }
}
