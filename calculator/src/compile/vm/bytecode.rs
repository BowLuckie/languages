use crate::{
    ast::{Node, Operator},
    compile::{
        Compile,
        vm::opcode::{Address, OpCode, make_op},
    },
};

#[derive(Default)]
pub struct Bytecode {
    pub instructions: Vec<u8>,
    pub constant: Vec<Node>,
}

pub struct Interpreter {
    bytecode: Bytecode,
}

impl Compile for Interpreter {
    type Output = Bytecode;

    fn from_ast(ast: Vec<Node>) -> Self::Output {
        let mut interpreter = Interpreter {
            bytecode: Bytecode::default(),
        };

        for node in ast {
            println!("compiling node {:?}", node);
            interpreter.interpret_node(node);
            interpreter.add_instruction(OpCode::OpPop);
        }

        interpreter.bytecode
    }
}

impl Interpreter {
    fn add_constant(&mut self, node: Node) -> Address {
        let address = self.bytecode.constant.len() as Address;
        self.bytecode.constant.push(node);
        address
    }

    fn add_instruction(&mut self, opcode: OpCode) -> Address {
        let address = self.bytecode.instructions.len() as Address;
        self.bytecode.instructions.extend(make_op(&opcode));
        println!(
            "added instructions {:?} from opcode {:?}",
            self.bytecode.instructions,
            opcode.clone()
        );
        address
    }

    fn interpret_node(&mut self, node: Node) {
        match node {
            Node::Int(n) => {
                let const_ptr = self.add_constant(Node::Int(n));
                self.add_instruction(OpCode::OpConstant(const_ptr));
            }
            Node::UnaryExpr { op, child } => {
                self.interpret_node(*child);
                match op {
                    Operator::Plus => self.add_instruction(OpCode::OpPlus),
                    Operator::Minus => self.add_instruction(OpCode::OpMinus),
                };
            }
            Node::BinaryExpr { op, lhs, rhs } => {
                self.interpret_node(*lhs);
                self.interpret_node(*rhs);
                match op {
                    Operator::Plus => self.add_instruction(OpCode::OpPlus),
                    Operator::Minus => self.add_instruction(OpCode::OpMinus),
                };
            }
        }
    }
}
