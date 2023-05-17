use bytes::{BufMut, Bytes, BytesMut};

/* All opcodes
Const        |   0
Null         |   1
True         |   2
False        |   3
SetGVar      |   4
GetGVar      |   5
Tuple        |   6
Set          |   7
TupleRn      |   8
SetRn        |   9
Pop          |  20
PushMatch    |  21
PopMatch     |  22
Jump         |  23
JumpNotTrue  |  24
JumpNotMatch |  25
Index        | 100
Range        | 101
Pick         | 102
Call         | 103
NullCoal     | 200
TupleStart   | 201
Exp          | 202
Mult         | 203
Inter        | 204
Div          | 205
Mod          | 206
IntDiv       | 207
Add          | 208
Subtract     | 209
With         | 210
Less         | 211
Union        | 212
In           | 213
Notin        | 214
Subset       | 215
Lt           | 216
Lteq         | 217
Eq           | 218
Neq          | 219
And          | 220
Or           | 221
Impl         | 222
Iff          | 223
Negate       | 226
DynVar       | 227
Size         | 228
Not          | 229
*/

pub trait OpCodeMake {
    fn make(self) -> Bytes;
    const OPERAND_COUNTS: &'static [usize] = &[];
}

pub trait OpCodeMakeWithU16 {
    fn make(self, operand: u16) -> Bytes;
    const OPERAND_COUNTS: &'static [usize] = &[2];
}

/** Used to implement all opcodes to give them a value */
pub trait OpCode {
    const VAL: u8;
}

/** Used to implement opcodes with no operands */
pub trait OpCodeNone {}

/** Used to implement opcodes which take a single 2-byte operand */
pub trait OpCodeU16 {}

impl<T: Sized + OpCode + OpCodeNone> OpCodeMake for T {
    fn make(self) -> Bytes {
        let mut bytes = BytesMut::with_capacity(1);
        bytes.put_u8(Self::VAL);
        bytes.freeze()
    }
}

impl<T: Sized + OpCode + OpCodeU16> OpCodeMakeWithU16 for T {
    fn make(self, operand: u16) -> Bytes {
        let mut bytes = BytesMut::with_capacity(1);
        bytes.put_u8(Self::VAL);
        bytes.put_u16(operand);
        bytes.freeze()
    }
}

#[derive(Debug)]
pub struct Const;
impl OpCodeU16 for Const {}
impl OpCode for Const {
    const VAL: u8 = 0;
}

#[derive(Debug)]
pub struct Null;
impl OpCodeNone for Null {}
impl OpCode for Null {
    const VAL: u8 = 1;
}

#[derive(Debug)]
pub struct True;
impl OpCodeNone for True {}
impl OpCode for True {
    const VAL: u8 = 2;
}

#[derive(Debug)]
pub struct False;
impl OpCodeNone for False {}
impl OpCode for False {
    const VAL: u8 = 3;
}

#[derive(Debug)]
pub struct SetGVar;
impl OpCodeU16 for SetGVar {}
impl OpCode for SetGVar {
    const VAL: u8 = 4;
}

#[derive(Debug)]
pub struct GetGVar;
impl OpCodeU16 for GetGVar {}
impl OpCode for GetGVar {
    const VAL: u8 = 5;
}

#[derive(Debug)]
pub struct Tuple;
impl OpCodeU16 for Tuple {}
impl OpCode for Tuple {
    const VAL: u8 = 6;
}

#[derive(Debug)]
pub struct Set;
impl OpCodeU16 for Set {}
impl OpCode for Set {
    const VAL: u8 = 7;
}

#[derive(Debug)]
pub struct TupleRn;
impl OpCodeU16 for TupleRn {}
impl OpCode for TupleRn {
    const VAL: u8 = 8;
}

#[derive(Debug)]
pub struct SetRn;
impl OpCodeU16 for SetRn {}
impl OpCode for SetRn {
    const VAL: u8 = 9;
}

#[derive(Debug)]
pub struct Pop;
impl OpCodeNone for Pop {}
impl OpCode for Pop {
    const VAL: u8 = 20;
}

#[derive(Debug)]
pub struct PushMatch;
impl OpCodeNone for PushMatch {}
impl OpCode for PushMatch {
    const VAL: u8 = 21;
}

#[derive(Debug)]
pub struct PopMatch;
impl OpCodeNone for PopMatch {}
impl OpCode for PopMatch {
    const VAL: u8 = 22;
}

#[derive(Debug)]
pub struct Jump;
impl OpCodeU16 for Jump {}
impl OpCode for Jump {
    const VAL: u8 = 23;
}

#[derive(Debug)]
pub struct JumpNotTrue;
impl OpCodeU16 for JumpNotTrue {}
impl OpCode for JumpNotTrue {
    const VAL: u8 = 24;
}

#[derive(Debug)]
pub struct JumpNotMatch;
impl OpCodeU16 for JumpNotMatch {}
impl OpCode for JumpNotMatch {
    const VAL: u8 = 25;
}

#[derive(Debug)]
pub struct Index;
impl OpCodeNone for Index {}
impl OpCode for Index {
    const VAL: u8 = 100;
}

#[derive(Debug)]
pub struct NullCoal;
impl OpCodeNone for NullCoal {}
impl OpCode for NullCoal {
    const VAL: u8 = 200;
}

#[derive(Debug)]
pub struct TupleStart;
impl OpCodeNone for TupleStart {}
impl OpCode for TupleStart {
    const VAL: u8 = 201;
}

#[derive(Debug)]
pub struct Exp;
impl OpCodeNone for Exp {}
impl OpCode for Exp {
    const VAL: u8 = 202;
}

#[derive(Debug)]
pub struct Mult;
impl OpCodeNone for Mult {}
impl OpCode for Mult {
    const VAL: u8 = 203;
}

#[derive(Debug)]
pub struct Inter;
impl OpCodeNone for Inter {}
impl OpCode for Inter {
    const VAL: u8 = 204;
}

#[derive(Debug)]
pub struct Div;
impl OpCodeNone for Div {}
impl OpCode for Div {
    const VAL: u8 = 205;
}

#[derive(Debug)]
pub struct Mod;
impl OpCodeNone for Mod {}
impl OpCode for Mod {
    const VAL: u8 = 206;
}

#[derive(Debug)]
pub struct IntDiv;
impl OpCodeNone for IntDiv {}
impl OpCode for IntDiv {
    const VAL: u8 = 207;
}

#[derive(Debug)]
pub struct Add;
impl OpCodeNone for Add {}
impl OpCode for Add {
    const VAL: u8 = 208;
}

#[derive(Debug)]
pub struct Subtract;
impl OpCodeNone for Subtract {}
impl OpCode for Subtract {
    const VAL: u8 = 209;
}

#[derive(Debug)]
pub struct With;
impl OpCodeNone for With {}
impl OpCode for With {
    const VAL: u8 = 210;
}

#[derive(Debug)]
pub struct Less;
impl OpCodeNone for Less {}
impl OpCode for Less {
    const VAL: u8 = 211;
}

#[derive(Debug)]
pub struct Union;
impl OpCodeNone for Union {}
impl OpCode for Union {
    const VAL: u8 = 212;
}

#[derive(Debug)]
pub struct In;
impl OpCodeNone for In {}
impl OpCode for In {
    const VAL: u8 = 213;
}

#[derive(Debug)]
pub struct Notin;
impl OpCodeNone for Notin {}
impl OpCode for Notin {
    const VAL: u8 = 214;
}

#[derive(Debug)]
pub struct Subset;
impl OpCodeNone for Subset {}
impl OpCode for Subset {
    const VAL: u8 = 215;
}

#[derive(Debug)]
pub struct Lt;
impl OpCodeNone for Lt {}
impl OpCode for Lt {
    const VAL: u8 = 216;
}

#[derive(Debug)]
pub struct Lteq;
impl OpCodeNone for Lteq {}
impl OpCode for Lteq {
    const VAL: u8 = 217;
}

#[derive(Debug)]
pub struct Eq;
impl OpCodeNone for Eq {}
impl OpCode for Eq {
    const VAL: u8 = 218;
}

#[derive(Debug)]
pub struct Neq;
impl OpCodeNone for Neq {}
impl OpCode for Neq {
    const VAL: u8 = 219;
}

#[derive(Debug)]
pub struct And;
impl OpCodeNone for And {}
impl OpCode for And {
    const VAL: u8 = 220;
}

#[derive(Debug)]
pub struct Or;
impl OpCodeNone for Or {}
impl OpCode for Or {
    const VAL: u8 = 221;
}

#[derive(Debug)]
pub struct Impl;
impl OpCodeNone for Impl {}
impl OpCode for Impl {
    const VAL: u8 = 222;
}

#[derive(Debug)]
pub struct Iff;
impl OpCodeNone for Iff {}
impl OpCode for Iff {
    const VAL: u8 = 223;
}

#[derive(Debug)]
pub struct Negate;
impl OpCodeNone for Negate {}
impl OpCode for Negate {
    const VAL: u8 = 226;
}

#[derive(Debug)]
pub struct DynVar;
impl OpCodeNone for DynVar {}
impl OpCode for DynVar {
    const VAL: u8 = 227;
}

#[derive(Debug)]
pub struct Size;
impl OpCodeNone for Size {}
impl OpCode for Size {
    const VAL: u8 = 228;
}

#[derive(Debug)]
pub struct Not;
impl OpCodeNone for Not {}
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
