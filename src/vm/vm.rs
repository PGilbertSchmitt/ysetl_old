use bytes::{Bytes, Buf};
use std::io::Cursor;

use crate::code::code::codes;
use crate::object::object::Object;
use crate::compiler::compiler::Bytecode;

const STACK_SIZE: usize = 2048;

#[derive(Debug)]
pub struct VM {
    instructions: Bytes,
    constants: Vec<Object>,

    stack: Vec<Object>,
}

impl VM {
    pub fn new(bytecode: Bytecode) -> Self {
        VM {
            instructions: bytecode.instuctions,
            constants: bytecode.constants,

            stack: Vec::with_capacity(STACK_SIZE),
        }
    }

    pub fn peek_top(&self) -> Option<&Object> {
        self.stack.last()
    }

    pub fn run(&mut self) -> Object {
        let mut c = Cursor::new(self.instructions.as_ref());
        while c.has_remaining() {
            let op = c.get_u8();
            match op {
                codes::CONST => {
                    self.stack.push(self.constants[c.get_u16() as usize]);
                },
                codes::ADD |
                codes::SUBTRACT => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();
                    self.stack.push(Object::math(left, right, op).unwrap());
                },
                _ => unimplemented!()
            }
        }
        self.stack.pop().unwrap()
    }
}
