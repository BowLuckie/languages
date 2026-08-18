use crate::{
    ast::{Node, Operator},
    compile::{
        Compile,
        vm::opcode::{Address, Byte, OpCode, make_byte},
    },
};

#[derive(Debug, Default)]
pub(super) struct Bytecode {
    pub(super) instructions: Vec<Byte>,
    pub(super) constant: Vec<Node>,
}

pub(super) struct VmInterpreter {
    bytecode: Bytecode,
}

impl Compile for VmInterpreter {
    type Output = Bytecode;

    fn from_ast(ast: Vec<Node>) -> Self::Output {
        let mut interpreter = VmInterpreter {
            bytecode: Bytecode::default(),
        };

        for node in ast {
            println!("compiling node {:?}", node);
            interpreter.interpret_node(node);
        }

        interpreter.bytecode
    }
}

impl VmInterpreter {
    fn add_constant(&mut self, node: Node) -> Address {
        let address = self.bytecode.constant.len() as Address;
        self.bytecode.constant.push(node);
        address
    }

    fn add_instruction(&mut self, opcode: OpCode) -> Address {
        let address = self.bytecode.instructions.len() as Address;
        self.bytecode.instructions.extend(make_byte(&opcode));
        address
    }

    fn interpret_node(&mut self, node: Node) {
        match node {
            Node::Int(n) => {
                let const_ptr = self.add_constant(Node::Int(n));
                self.add_instruction(OpCode::LDC(const_ptr));
            }

            Node::UnaryExpr { op, child } => {
                self.interpret_node(*child);
                match op {
                    Operator::Plus => self.add_instruction(OpCode::POS),
                    Operator::Min => self.add_instruction(OpCode::NEG),
                    Operator::Mul => todo!(),
                    Operator::Div => todo!(),
                };
            }

            Node::BinaryExpr { op, lhs, rhs } => {
                self.interpret_node(*lhs);
                self.interpret_node(*rhs);
                match op {
                    Operator::Plus => self.add_instruction(OpCode::ADD),
                    Operator::Min => self.add_instruction(OpCode::SUB),
                    Operator::Mul => todo!(),
                    Operator::Div => todo!(),
                };
            }
            Node::Float(_) => todo!(),
        }
    }
}
