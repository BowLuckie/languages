#[derive(Clone, Debug)]
pub enum OpCode {
    OpConstant(Address), // pointer
    OpPop,
    OpAdd,
    OpSub,
    OpPlus,
    OpMinus,
}

pub type Address = u16;
pub type Byte = u8;

pub fn make_op(opcode: &OpCode) -> Vec<u8> {
    match opcode {
        OpCode::OpConstant(arg) => [vec![0x01], bytes_of_address(*arg).into()].concat(),
        OpCode::OpPop => vec![0x02],
        OpCode::OpAdd => vec![0x03],
        OpCode::OpSub => vec![0x04],
        OpCode::OpPlus => vec![0x0A],
        OpCode::OpMinus => vec![0x0B],
    }
}

fn bytes_of_address(address: Address) -> [Byte; 2] {
    [(address >> 8) as Byte, address as Byte]
}
