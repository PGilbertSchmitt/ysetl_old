use bytes::{Bytes, Buf};
use super::code::*;

pub fn lookup(byte: u8) -> Option<(&'static [usize], &'static str)> {
    match byte {
        Const::VAL => Some((Const::OPERAND_COUNTS, "Const")),
        Null::VAL => Some((Null::OPERAND_COUNTS, "Null")),
        True::VAL => Some((True::OPERAND_COUNTS, "True")),
        False::VAL => Some((False::OPERAND_COUNTS, "False")),
        SetGVar::VAL => Some((SetGVar::OPERAND_COUNTS, "SetGVar")),
        GetGVar::VAL => Some((GetGVar::OPERAND_COUNTS, "GetGVar")),
        Pop::VAL => Some((Pop::OPERAND_COUNTS, "Pop")),
        PushMatch::VAL => Some((PushMatch::OPERAND_COUNTS, "PushMatch")),
        PopMatch::VAL => Some((PopMatch::OPERAND_COUNTS, "PopMatch")),
        Jump::VAL => Some((Jump::OPERAND_COUNTS, "Jump")),
        JumpNotTrue::VAL => Some((JumpNotTrue::OPERAND_COUNTS, "JumpNotTrue")),
        JumpNotMatch::VAL => Some((JumpNotMatch::OPERAND_COUNTS, "JumpNotMatch")),
        NullCoal::VAL => Some((NullCoal::OPERAND_COUNTS, "NullCoal")),
        TupleStart::VAL => Some((TupleStart::OPERAND_COUNTS, "TupleStart")),
        Exp::VAL => Some((Exp::OPERAND_COUNTS, "Exp")),
        Mult::VAL => Some((Mult::OPERAND_COUNTS, "Mult")),
        Inter::VAL => Some((Inter::OPERAND_COUNTS, "Inter")),
        Div::VAL => Some((Div::OPERAND_COUNTS, "Div")),
        Mod::VAL => Some((Mod::OPERAND_COUNTS, "Mod")),
        IntDiv::VAL => Some((IntDiv::OPERAND_COUNTS, "IntDiv")),
        Add::VAL => Some((Add::OPERAND_COUNTS, "Add")),
        Subtract::VAL => Some((Subtract::OPERAND_COUNTS, "Subtract")),
        With::VAL => Some((With::OPERAND_COUNTS, "With")),
        Less::VAL => Some((Less::OPERAND_COUNTS, "Less")),
        Union::VAL => Some((Union::OPERAND_COUNTS, "Union")),
        In::VAL => Some((In::OPERAND_COUNTS, "In")),
        Notin::VAL => Some((Notin::OPERAND_COUNTS, "Notin")),
        Subset::VAL => Some((Subset::OPERAND_COUNTS, "Subset")),
        Lt::VAL => Some((Lt::OPERAND_COUNTS, "Lt")),
        Lteq::VAL => Some((Lteq::OPERAND_COUNTS, "Lteq")),
        Eq::VAL => Some((Eq::OPERAND_COUNTS, "Eq")),
        Neq::VAL => Some((Neq::OPERAND_COUNTS, "Neq")),
        And::VAL => Some((And::OPERAND_COUNTS, "And")),
        Or::VAL => Some((Or::OPERAND_COUNTS, "Or")),
        Impl::VAL => Some((Impl::OPERAND_COUNTS, "Impl")),
        Iff::VAL => Some((Iff::OPERAND_COUNTS, "Iff")),
        Negate::VAL => Some((Negate::OPERAND_COUNTS, "Negate")),
        DynVar::VAL => Some((DynVar::OPERAND_COUNTS, "DynVar")),
        Size::VAL => Some((Size::OPERAND_COUNTS, "Size")),
        Not::VAL => Some((Not::OPERAND_COUNTS, "Not")),
        other => {
            println!("No idea how to print code: {}", other);
            None
        }
    }
}

pub fn print_bytes(bytes: &Bytes) -> String {
    let len = bytes.len();
    let mut buf = bytes.as_ref();

    let mut parts: Vec<String> = vec![];
    while buf.remaining() > 0 {
        let pos = len - buf.remaining();
        parts.push(print_op(&mut buf, pos));
    }

    parts.join("\n")
}

fn print_op(buf: &mut dyn Buf, pos: usize) -> String {
    let code_byte = buf.get_u8();
    let (sizes, name) = lookup(code_byte).unwrap();

    let mut output = format!("{:>4}: ", pos.to_string());
    output.push_str(name);

    for size in sizes.iter() {
        if *size == 0 { break }
        match *size {
            0 => break,
            2 => {
                output.push_str(&format!(" {}", buf.get_u16()))
            },
            _ => unreachable!(),
        }
    }

    output
}
