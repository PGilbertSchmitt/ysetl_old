use bytes::{BufMut, Bytes, BytesMut};
use std::collections::HashMap;

pub type OpCode = u8;

pub struct Def(pub [usize; 2], pub &'static str);

pub mod codes {
    use super::OpCode;

    pub const CONST:OpCode=0;
    pub const ADD:OpCode=1;
    pub const SUBTRACT:OpCode=2;
}

pub trait Op {
    fn lookup(&self) -> &'static Def;
    fn make(self) -> Bytes;
    fn make_with(self, operands: &[usize]) -> Bytes;
}

lazy_static::lazy_static! {
    pub static ref DEFINITIONS: HashMap<u8, Def> = {
        HashMap::from([
            (codes::CONST,    Def([2, 0], "Push Const")),
            (codes::ADD,      Def([0, 0], "OpAdd")),
            (codes::SUBTRACT, Def([0, 0], "OpSubtract")),
        ])
    };
    static ref EMPTY_SIZES: [usize; 2] = [0, 0];
}

impl Op for OpCode {
    /** Lookup the name and sizes of operands for an opcode */
    fn lookup(&self) -> &'static Def {
        DEFINITIONS.get(self).unwrap()
    }

    fn make(self) -> Bytes {
        let &Def(sizes, _) = DEFINITIONS.get(&self).unwrap();
        if sizes != [0, 0] {
            panic!("Cannot use make for op codes with arguments")
        };
        let mut bytes = BytesMut::with_capacity(1);
        bytes.put_u8(self);
        bytes.freeze()
    }

    fn make_with(self, operands: &[usize]) -> Bytes {
        let &Def(sizes, _) = DEFINITIONS.get(&self).unwrap();
        let ins_len: usize = 1 + sizes.iter().sum::<usize>();
        let mut instruction = BytesMut::with_capacity(ins_len);
        instruction.put_u8(self);

        for (i, operand) in operands.iter().enumerate() {
            let size = sizes[i];
            match size {
                0 => panic!("Too many operands provided to opcode {:?}", &self),
                1 => instruction.put_u8(*operand as u8),
                2 => instruction.put_u16(*operand as u16),
                _ => panic!("Can't add operand with size {size}"),
            }
        }

        instruction.freeze()
    }
}

#[cfg(test)]
mod tests {
    use super::{Op,codes};

    #[test]
    fn op_const() {
        assert_eq!(&codes::CONST.make_with(&[5])[..], [0, 0, 5]);
        assert_eq!(&codes::CONST.make_with(&[258])[..], [0, 1, 2]);
    }

    #[test]
    fn op_add() {
        assert_eq!(&codes::ADD.make()[..], [1]);
    }
}
