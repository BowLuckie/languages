use std::fmt::{self, Display, Formatter};

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

use super::bytecode::VmInterpreter;

const STACK_SIZE: usize = usize::pow(2, 9); // 2^9

pub struct VM {
    bytecode: Bytecode,
    stack: [Node; STACK_SIZE],
    top: usize,
}

impl Compile for VM {
    type Output = Vec<Node>;

    fn from_ast(ast: Vec<Node>) -> Self::Output {
        let bytecode = VmInterpreter::from_ast(ast);
        println!("\n==== bytecode ====\n{}", bytecode);
        let mut vm = VM::new(bytecode);
        vm.run();
        let mut results = vec![];
        while let Some(val) = vm.result() {
            results.push(val);
            vm.pop();
        }
        results.reverse();
        results
    }
}

impl VM {
    fn new(bytecode: Bytecode) -> Self {
        Self {
            bytecode,
            stack: std::array::from_fn(|_| Node::Int(0)),
            top: 0,
        }
    }

    fn run(&mut self) {
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
                        self.push(Node::Int(l + r));
                    } else {
                        panic!(
                            "improper arguments to add instruction OpAdd (0x03) {}",
                            instruction_addr,
                        );
                    }
                }

                0x04 => {
                    if let (Node::Int(r), Node::Int(l)) = (self.pop(), self.pop()) {
                        self.push(Node::Int(l - r));
                    } else {
                        panic!(
                            "improper arguments to add instruction OpSub (0x04) {}",
                            instruction_addr,
                        );
                    }
                }

                0x0A => {
                    if let Node::Int(n) = self.pop() {
                        self.push(Node::Int(-n));
                    } else {
                        panic!(
                            "improper arguments to add instruction OpNeg (0x0A) {}",
                            instruction_addr,
                        );
                    }
                }

                0x0B => {
                    if !matches!(self.peek_top(), Node::Int(_)) {
                        panic!(
                            "improper arguments to add instruction OpPos (0x0B) {}",
                            instruction_addr,
                        );
                    }
                }
                _ => panic!("unknown instruction! {}", op),
            }
        }
    }

    fn push(&mut self, node: Node) {
        self.stack[self.top] = node;
        if self.top >= 512 {
            panic!("stack overflow! out of memory."); // 
        }

        self.top = unsafe { self.top.unchecked_add(1) };
    }

    fn pop(&mut self) -> Node {
        self.top -= 1;
        self.stack[self.top].clone() // TODO: mem::replace()
    }

    fn peek_top(&self) -> Node {
        self.stack[self.top - 1].clone()
    }

    fn result(&self) -> Option<Node> {
        if self.top > 0 {
            Some(self.stack[self.top - 1].clone())
        } else {
            None
        }
    }
}

impl Display for Bytecode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let instructions = &self.instructions;
        let mut i = 0;

        writeln!(f, "Instructions:")?;

        while i < instructions.len() {
            match instructions[i] {
                0x01 => {
                    if i + 2 >= instructions.len() {
                        writeln!(f, "{i:04X}: LDC <missing address>")?;
                        break;
                    }

                    let address = u16::from_be_bytes([instructions[i + 1], instructions[i + 2]]);

                    writeln!(f, "{i:04X}: LDC 0x{address:04X}")?;
                    i += 3;
                }

                0x02 => {
                    writeln!(f, "{i:04X}: POP")?;
                    i += 1;
                }

                0x03 => {
                    writeln!(f, "{i:04X}: ADD")?;
                    i += 1;
                }

                0x04 => {
                    writeln!(f, "{i:04X}: SUB")?;
                    i += 1;
                }

                0x0A => {
                    writeln!(f, "{i:04X}: NEG")?;
                    i += 1;
                }

                0x0B => {
                    writeln!(f, "{i:04X}: POS")?;
                    i += 1;
                }

                byte => {
                    writeln!(f, "{i:04X}: UNKNOWN 0x{byte:02X}")?;
                    i += 1;
                }
            }
        }

        writeln!(f, "\nConstants:")?;

        for (i, constant) in self.constant.iter().enumerate() {
            writeln!(f, "{i:04X}: {constant}")?;
        }

        Ok(())
    }
}
