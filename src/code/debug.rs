use bytes::{Bytes, Buf};
use super::code::lookup;

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
