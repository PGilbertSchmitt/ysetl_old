use bytes::{BufMut, Bytes, BytesMut};

pub trait OpCodeMake {
    fn make(self) -> Bytes;
    const OPERAND_COUNTS: &'static [usize] = &[];
}

pub trait OpCodeMakeWithU16 {
    fn make(self, operand: u16) -> Bytes;
    const OPERAND_COUNTS: &'static [usize] = &[2];
}

/** Used to implement opcodes with no operands */
pub trait OpCode {
    const VAL: u8;
}

/** Used to implement opcodes which take a single 2-byte operand */
pub trait OpCodeU16 {
    const VAL: u8;
}

impl<T: Sized + OpCode> OpCodeMake for T {
    fn make(self) -> Bytes {
        let mut bytes = BytesMut::with_capacity(1);
        bytes.put_u8(Self::VAL);
        bytes.freeze()
    }
}

impl<T: Sized + OpCodeU16> OpCodeMakeWithU16 for T {
    fn make(self, operand: u16) -> Bytes {
        let mut bytes = BytesMut::with_capacity(1);
        bytes.put_u8(Self::VAL);
        bytes.put_u16(operand);
        bytes.freeze()
    }
}

#[derive(Debug)]
pub struct Const;
impl OpCodeU16 for Const {
    const VAL: u8 = 0;
}

#[derive(Debug)]
pub struct Null;
impl OpCode for Null {
    const VAL: u8 = 1;
}

#[derive(Debug)]
pub struct True;
impl OpCode for True {
    const VAL: u8 =  2;
}
#[derive(Debug)]
pub struct False;
impl OpCode for False {
    const VAL: u8 =  3;
}
#[derive(Debug)]
pub struct SetGVar;
impl OpCodeU16 for SetGVar {
    const VAL: u8 =  4;
}
#[derive(Debug)]
pub struct GetGVar;
impl OpCodeU16 for GetGVar {
    const VAL: u8 =  5;
}

#[derive(Debug)]
pub struct Pop;
impl OpCode for Pop {
    const VAL: u8 = 20;
}

#[derive(Debug)]
pub struct PushMatch;
impl OpCode for PushMatch {
    const VAL: u8 = 21;
}

#[derive(Debug)]
pub struct PopMatch;
impl OpCode for PopMatch {
    const VAL: u8 = 22;
}

#[derive(Debug)]
pub struct Jump;
impl OpCodeU16 for Jump {
    const VAL: u8 = 23;
}

#[derive(Debug)]
pub struct JumpNotTrue;
impl OpCodeU16 for JumpNotTrue {
    const VAL: u8 = 24;
}

#[derive(Debug)]
pub struct JumpNotMatch;
impl OpCodeU16 for JumpNotMatch {
    const VAL: u8 = 25;
}

#[derive(Debug)]
pub struct NullCoal;
impl OpCode for NullCoal {
    const VAL: u8 = 200;
}

#[derive(Debug)]
pub struct TupleStart;
impl OpCode for TupleStart {
    const VAL: u8 = 201;
}

#[derive(Debug)]
pub struct Exp;
impl OpCode for Exp {
    const VAL: u8 = 202;
}

#[derive(Debug)]
pub struct Mult;
impl OpCode for Mult {
    const VAL: u8 = 203;
}

#[derive(Debug)]
pub struct Inter;
impl OpCode for Inter {
    const VAL: u8 = 204;
}

#[derive(Debug)]
pub struct Div;
impl OpCode for Div {
    const VAL: u8 = 205;
}

#[derive(Debug)]
pub struct Mod;
impl OpCode for Mod {
    const VAL: u8 = 206;
}

#[derive(Debug)]
pub struct IntDiv;
impl OpCode for IntDiv {
    const VAL: u8 = 207;
}

#[derive(Debug)]
pub struct Add;
impl OpCode for Add {
    const VAL: u8 = 208;
}

#[derive(Debug)]
pub struct Subtract;
impl OpCode for Subtract {
    const VAL: u8 = 209;
}

#[derive(Debug)]
pub struct With;
impl OpCode for With {
    const VAL: u8 = 210;
}

#[derive(Debug)]
pub struct Less;
impl OpCode for Less {
    const VAL: u8 = 211;
}

#[derive(Debug)]
pub struct Union;
impl OpCode for Union {
    const VAL: u8 = 212;
}

#[derive(Debug)]
pub struct In;
impl OpCode for In {
    const VAL: u8 = 213;
}

#[derive(Debug)]
pub struct Notin;
impl OpCode for Notin {
    const VAL: u8 = 214;
}

#[derive(Debug)]
pub struct Subset;
impl OpCode for Subset {
    const VAL: u8 = 215;
}

#[derive(Debug)]
pub struct Lt;
impl OpCode for Lt {
    const VAL: u8 = 216;
}

#[derive(Debug)]
pub struct Lteq;
impl OpCode for Lteq {
    const VAL: u8 = 217;
}

#[derive(Debug)]
pub struct Eq;
impl OpCode for Eq {
    const VAL: u8 = 218;
}

#[derive(Debug)]
pub struct Neq;
impl OpCode for Neq {
    const VAL: u8 = 219;
}

#[derive(Debug)]
pub struct And;
impl OpCode for And {
    const VAL: u8 = 220;
}

#[derive(Debug)]
pub struct Or;
impl OpCode for Or {
    const VAL: u8 = 221;
}

#[derive(Debug)]
pub struct Impl;
impl OpCode for Impl {
    const VAL: u8 = 222;
}

#[derive(Debug)]
pub struct Iff;
impl OpCode for Iff {
    const VAL: u8 = 223;
}

#[derive(Debug)]
pub struct Negate;
impl OpCode for Negate {
    const VAL: u8 = 226;
}

#[derive(Debug)]
pub struct DynVar;
impl OpCode for DynVar {
    const VAL: u8 = 227;
}

#[derive(Debug)]
pub struct Size;
impl OpCode for Size {
    const VAL: u8 = 228;
}

#[derive(Debug)]
pub struct Not;
impl OpCode for Not {
    const VAL: u8 = 229;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn op_const() {
        assert_eq!(Const.make(5)[..], [Const::VAL, 0, 5]);
        assert_eq!(Const.make(258)[..], [Const::VAL, 1, 2]);
    }

    #[test]
    fn op_add() {
        assert_eq!(Add.make()[..], [Add::VAL]);
    }
}
