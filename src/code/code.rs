use bytes::{BufMut, Bytes, BytesMut};
use std::collections::HashMap;

pub type OpCode = u8;

pub struct Def(pub [usize; 2], pub &'static str);

pub mod codes {
    use super::OpCode;

    // Values
    pub const CONST: OpCode = 0;
    pub const NULL: OpCode = 1;
    pub const TRUE: OpCode = 2;
    pub const FALSE: OpCode = 3;

    // Control
    pub const POP: OpCode = 20;
    pub const JUMP: OpCode = 21;
    pub const JUMP_NOT_TRUE: OpCode = 22;

    // Binops 200-225
    pub const NULL_COAL: OpCode = 200;
    pub const TUPLE_START: OpCode = 201;
    pub const EXP: OpCode = 202;
    pub const MULT: OpCode = 203;
    pub const INTER: OpCode = 204;
    pub const DIV: OpCode = 205;
    pub const MOD: OpCode = 206;
    pub const INT_DIV: OpCode = 207;
    pub const ADD: OpCode = 208;
    pub const SUBTRACT: OpCode = 209;
    pub const WITH: OpCode = 210;
    pub const LESS: OpCode = 211;
    pub const UNION: OpCode = 212;
    pub const IN: OpCode = 213;
    pub const NOTIN: OpCode = 214;
    pub const SUBSET: OpCode = 215;
    pub const LT: OpCode = 216;
    pub const LTEQ: OpCode = 217;
    pub const EQ: OpCode = 218;
    pub const NEQ: OpCode = 219;
    pub const AND: OpCode = 220;
    pub const OR: OpCode = 221;
    pub const IMPL: OpCode = 222;
    pub const IFF: OpCode = 223;

    // Preops 
    pub const NEGATE: OpCode = 226;
    pub const DYN_VAR: OpCode = 227;
    pub const SIZE: OpCode = 228;
    pub const NOT: OpCode = 229;
}

pub trait Op {
    fn lookup(&self) -> &'static Def;
    fn make(self) -> Bytes;
    fn make_with(self, operands: &[usize]) -> Bytes;
}

lazy_static::lazy_static! {
    pub static ref DEFINITIONS: HashMap<u8, Def> = {
        HashMap::from([
            (codes::CONST,         Def([2, 0], "Push Const")),
            (codes::NULL,          Def([0, 0], "Push Null")),
            (codes::TRUE,          Def([0, 0], "Push True")),
            (codes::FALSE,         Def([0, 0], "Push False")),

            (codes::POP,           Def([0, 0], "Pop")),
            (codes::JUMP,          Def([2, 0], "Jump")),
            (codes::JUMP_NOT_TRUE, Def([2, 0], "JumpIfNotTrue")),

            (codes::NULL_COAL,     Def([0, 0], "OpNullCoal")),
            (codes::TUPLE_START,   Def([0, 0], "OpTupleStart")),
            (codes::EXP,           Def([0, 0], "OpExp")),
            (codes::MULT,          Def([0, 0], "OpMult")),
            (codes::INTER,         Def([0, 0], "OpInter")),
            (codes::DIV,           Def([0, 0], "OpDiv")),
            (codes::MOD,           Def([0, 0], "OpMod")),
            (codes::INT_DIV,       Def([0, 0], "OpIntDiv")),
            (codes::ADD,           Def([0, 0], "OpAdd")),
            (codes::SUBTRACT,      Def([0, 0], "OpSubtract")),
            (codes::WITH,          Def([0, 0], "OpWith")),
            (codes::LESS,          Def([0, 0], "OpLess")),
            (codes::UNION,         Def([0, 0], "OpUnion")),
            (codes::IN,            Def([0, 0], "OpIn")),
            (codes::NOTIN,         Def([0, 0], "OpNotin")),
            (codes::SUBSET,        Def([0, 0], "OpSubset")),
            (codes::LT,            Def([0, 0], "OpLT")),
            (codes::LTEQ,          Def([0, 0], "OpLTEQ")),
            (codes::EQ,            Def([0, 0], "OpEQ")),
            (codes::NEQ,           Def([0, 0], "OpNEQ")),
            (codes::AND,           Def([0, 0], "OpAnd")),
            (codes::OR,            Def([0, 0], "OpOr")),
            (codes::IMPL,          Def([0, 0], "OpImpl")),
            (codes::IFF,           Def([0, 0], "OpIff")),

            (codes::NEGATE,        Def([0, 0], "OpNegate")),
            (codes::DYN_VAR,       Def([0, 0], "OpDynVar")),
            (codes::SIZE,          Def([0, 0], "OpSize")),
            (codes::NOT,           Def([0, 0], "OpNot")),
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
    use super::{codes, Op};

    #[test]
    fn op_const() {
        assert_eq!(&codes::CONST.make_with(&[5])[..], [0, 0, 5]);
        assert_eq!(&codes::CONST.make_with(&[258])[..], [0, 1, 2]);
    }

    #[test]
    fn op_add() {
        assert_eq!(&codes::ADD.make()[..], [codes::ADD]);
    }
}
