use bytes::{Buf, Bytes};
use std::io::Cursor;

use crate::code::code::codes;
use crate::compiler::compiler::Bytecode;
use crate::object::object::Object;

const STACK_SIZE: usize = 2048;

trait Stack {
    /** Pops last two objects off the stack, and returns them in the order they're removed */
    fn pop_two(&mut self) -> (Object, Object);
}

impl Stack for Vec<Object> {
    fn pop_two(&mut self) -> (Object, Object) {
        (self.pop().unwrap(), self.pop().unwrap())
    }
}

#[derive(Debug)]
pub struct VM {
    instructions: Bytes,
    constants: Vec<Object>,

    stack: Vec<Object>,
    last_pop: Object,
}

impl VM {
    pub fn new(bytecode: Bytecode) -> Self {
        VM {
            instructions: bytecode.instuctions,
            constants: bytecode.constants,

            stack: Vec::with_capacity(STACK_SIZE),
            last_pop: Object::Null,
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
                codes::CONST => self.stack.push(self.constants[c.get_u16() as usize]),
                codes::NULL => self.stack.push(Object::Null),
                codes::TRUE => self.stack.push(Object::True),
                codes::FALSE => self.stack.push(Object::False),

                codes::POP => {
                    self.last_pop = self.stack.pop().unwrap();
                }

                codes::ADD | codes::SUBTRACT => {
                    let (right, left) = self.stack.pop_two();
                    self.stack
                        .push(Object::numeric_math(left, right, op).unwrap());
                }
                codes::EQ => {
                    let (right, left) = self.stack.pop_two();
                    self.stack.push(if left == right {
                        Object::True
                    } else {
                        Object::False
                    });
                }
                codes::NEQ => {
                    let (right, left) = self.stack.pop_two();
                    self.stack.push(if left != right {
                        Object::True
                    } else {
                        Object::False
                    });
                }
                code => unimplemented!("Don't know how to execute code {code}"),
            }
        }

        self.last_pop
    }
}
