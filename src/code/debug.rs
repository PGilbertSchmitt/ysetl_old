use bytes::{BytesMut, Buf};
use super::code::{DEFINITIONS, Def};

pub fn print_bytes(bytes: &BytesMut) {
    // bytes.as_re
    let mut buf = bytes.as_ref();

    let mut parts: Vec<String> = vec![];
    while buf.remaining() > 0 {
        parts.push(print_op(&mut buf));
    }

    println!("{}", parts.join("\n"));
}

fn print_op(buf: &mut dyn Buf) -> String {
    let code_byte = buf.get_u8();
    let &Def(sizes, name) = DEFINITIONS.get(&code_byte).expect(&format!(
        "Error reading bytes, found unexpected op byte {}",
        &code_byte
    ));

    let mut output = String::from(name);

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
