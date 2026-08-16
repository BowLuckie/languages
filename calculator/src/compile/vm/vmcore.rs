use crate::{
    ast::Node,
    compile::{
        Compile,
        vm::{
            bytecode::Bytecode,
            opcode::{Byte, bytes_to_address},
        },
    },
};

const STACK_SIZE: usize = usize::pow(2, 9); // 2^9

pub struct VM {
    pub bytecode: Bytecode,
    stack: [Node; STACK_SIZE],
    top: usize,
}

impl Compile for VM {
    type Output = VM;

    fn from_ast(ast: Vec<Node>) -> Self::Output {
        let bytecode = super::bytecode::Interpreter::from_ast(ast);
        VM::new(bytecode)
    }
}

impl VM {
    pub fn new(bytecode: Bytecode) -> Self {
        Self {
            bytecode,
            stack: std::array::from_fn(|_| Node::Int(0)),
            top: 0,
        }
    }

    pub fn run(&mut self) {
        let mut ip = 0;
        while ip < self.bytecode.instructions.len() {
            let instruction_addr = ip;
            ip += 1;

            let op: Byte = self.bytecode.instructions[instruction_addr]; // hex of bytecode

            // OpCode::OpConstant(arg) => [vec![0x01], bytes_of_address(*arg).into()].concat(),
            // OpCode::OpPop => vec![0x02],
            // OpCode::OpAdd => vec![0x03],
            // OpCode::OpSub => vec![0x04],
            // OpCode::OpPlus => vec![0x0A],
            // OpCode::OpMinus => vec![0x0B],
            match op {
                0x01 => {
                    // OpConst
                    debug_assert!(instruction_addr + 2 < self.bytecode.instructions.len());
                    let bytes = unsafe {
                        [
                            *self
                                .bytecode
                                .instructions
                                .get_unchecked(instruction_addr + 1),
                            *self
                                .bytecode
                                .instructions
                                .get_unchecked(instruction_addr + 2),
                        ]
                    };

                    let address = bytes_to_address(bytes);
                    ip += 2;
                    let node = self.bytecode.constant[address as usize].clone();
                    self.push(node);
                }

                0x02 => {
                    self.top -= 1;
                }

                0x03 => {
                    if let (Node::Int(r), Node::Int(l)) = (self.pop(), self.pop()) {
                        return self.push(Node::Int(l + r));
                    }

                    panic!(
                        "improper arguments to add instruction OpAdd (0x05) {}",
                        instruction_addr,
                    );
                }

                0x04 => {
                    if let (Node::Int(r), Node::Int(l)) = (self.pop(), self.pop()) {
                        return self.push(Node::Int(l - r));
                    }

                    panic!(
                        "improper arguments to add instruction OpSub (0x04) {}",
                        instruction_addr,
                    );
                }

                0x0A => {
                    if !matches!(self.peek_top(), Node::Int(_)) {
                        panic!(
                            "improper arguments to add instruction OpPlus (0x05) {}",
                            instruction_addr,
                        );
                    }
                }

                0x0B => {
                    if let Node::Int(n) = self.pop() {
                        self.push(Node::Int(-n));
                    }
                }
                _ => panic!("unknown instruction! {}", op),
            }
        }
    }

    pub fn push(&mut self, node: Node) {
        self.stack[self.top] = node;
        if self.top >= 512 {
            panic!("stack overflow! out of memory."); // 
        }

        self.top = unsafe { self.top.unchecked_add(1) };
    }

    pub fn pop(&mut self) -> Node {
        self.top -= 1;
        self.stack[self.top].clone() // TODO: mem::replace()
    }

    pub fn peek_top(&self) -> Node {
        self.stack[self.top - 1].clone()
    }

    pub fn result(&self) -> Option<Node> {
        if self.top > 0 {
            Some(self.stack[self.top - 1].clone())
        } else {
            None
        }
    }
}
