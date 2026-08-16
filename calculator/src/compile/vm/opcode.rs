#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(super) enum OpCode {
    LDC(Address),
    POP,
    ADD,
    SUB,
    NEG,
    POS,
}

pub(super) type Address = u16;
pub(super) type Byte = u8;

pub(super) fn make_byte(opcode: &OpCode) -> Vec<Byte> {
    match opcode {
        OpCode::LDC(arg) => [vec![0x01], bytes_of_address(*arg).into()].concat(),
        OpCode::POP => vec![0x02],
        OpCode::ADD => vec![0x03],
        OpCode::SUB => vec![0x04],
        OpCode::NEG => vec![0x0A],
        OpCode::POS => vec![0x0B],
    }
}

fn bytes_of_address(address: Address) -> [Byte; 2] {
    [(address >> 8) as Byte, address as Byte]
}

pub(super) fn bytes_to_address(bytes: [Byte; 2]) -> Address {
    ((bytes[0] as Address) << 8) | (bytes[1] as Address)
}
