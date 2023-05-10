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
                    println!("Last pop: {:?}", self.last_pop);
                }

                codes::ADD |
                codes::SUBTRACT |
                codes::MULT |
                codes::DIV |
                codes::INT_DIV |
                codes::EXP => {
                    let (right, left) = self.stack.pop_two();
                    self.stack
                        .push(Object::math(left, right, op).unwrap());
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

#[cfg(test)]
mod tests {
    use super::VM;
    use crate::compiler::compiler::Compiler;
    use crate::parser::parser;
    use crate::object::object::Object::*;

    fn vm_from(input: &str) -> VM {
        let mut c = Compiler::new();
        c.compile_expr(parser::parse_from_expr(input).unwrap());
        VM::new(c.finish())
    }

    #[test]
    fn op_const() {
        let mut vm = vm_from("99");
        vm.run();
        assert_eq!(vm.peek_top(), Some(&Integer(99)));
    }

    #[test]
    fn op_keyword_literals() {
        let mut vm = vm_from("true");
        vm.run();
        assert_eq!(vm.peek_top(), Some(&True));

        let mut vm = vm_from("false");
        vm.run();
        assert_eq!(vm.peek_top(), Some(&False));

        let mut vm = vm_from("null");
        vm.run();
        assert_eq!(vm.peek_top(), Some(&Null));
    }

    #[test]
    fn math_ops() {
        let mut vm = vm_from("3 + 4");
        vm.run();
        assert_eq!(vm.peek_top(), Some(&Integer(7)));
        
        let mut vm = vm_from("3 - 4");
        vm.run();
        assert_eq!(vm.peek_top(), Some(&Integer(-1)));
        
        let mut vm = vm_from("3.0 * 4");
        vm.run();
        assert_eq!(vm.peek_top(), Some(&Float(12.0)));
        
        let mut vm = vm_from("4 / 2");
        vm.run();
        assert_eq!(vm.peek_top(), Some(&Float(2.0)));
        
        let mut vm = vm_from("4 div 2");
        vm.run();
        assert_eq!(vm.peek_top(), Some(&Integer(2)));
        
        let mut vm = vm_from("4 ** 2");
        vm.run();
        assert_eq!(vm.peek_top(), Some(&Integer(16)));

    }
}
