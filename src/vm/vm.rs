use bytes::{Buf, Bytes};
use std::io::Cursor;

use crate::code::code::{self, OpCode, OpCodeU16};
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
    match_stack: Vec<Object>,

    stack: Vec<Object>,
    last_pop: Object,
}

impl VM {
    pub fn new(bytecode: Bytecode) -> Self {
        VM {
            instructions: bytecode.instuctions,
            constants: bytecode.constants,
            match_stack: Vec::new(),

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
                code::Const::VAL => self.stack.push(self.constants[c.get_u16() as usize]),
                code::Null::VAL => self.stack.push(Object::Null),
                code::True::VAL => self.stack.push(Object::True),
                code::False::VAL => self.stack.push(Object::False),

                code::Pop::VAL => {
                    self.last_pop = self.stack.pop().expect("Called pop on empty stack");
                    println!("Last pop: {:?}", self.last_pop);
                }

                code::Add::VAL
                | code::Subtract::VAL
                | code::Mult::VAL
                | code::Div::VAL
                | code::IntDiv::VAL
                | code::Exp::VAL
                | code::Lt::VAL
                | code::Lteq::VAL => {
                    let (right, left) = self.stack.pop_two();
                    self.stack.push(Object::math(left, right, op).unwrap());
                }
                code::Eq::VAL => {
                    let (right, left) = self.stack.pop_two();
                    self.stack.push(if left == right {
                        Object::True
                    } else {
                        Object::False
                    });
                }
                code::Neq::VAL => {
                    let (right, left) = self.stack.pop_two();
                    self.stack.push(if left != right {
                        Object::True
                    } else {
                        Object::False
                    });
                }
                
                code::Negate::VAL => {
                    let val = self.stack.pop().unwrap();
                    self.stack.push(val.negate());
                }
                code::Not::VAL => {
                    let val = self.stack.pop().unwrap();
                    self.stack.push(val.not());
                }

                code::Jump::VAL => {
                    let ptr = c.get_u16();
                    c.set_position(ptr as u64);
                }

                code::JumpNotTrue::VAL => {
                    let ptr = c.get_u16();
                    let top = self.stack.pop().unwrap();
                    if !top.truthy() {
                        c.set_position(ptr as u64);
                    }
                }

                code::PushMatch::VAL => {
                    let val = self.stack.pop().unwrap();
                    self.match_stack.push(val);
                }

                code::PopMatch::VAL => {
                    self.match_stack.pop();
                }

                code::JumpNotMatch::VAL => {
                    let ptr = c.get_u16();
                    let top = self.stack.pop();
                    if top.as_ref() != self.match_stack.last() {
                        c.set_position(ptr as u64);
                    }
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
    use crate::object::object::Object::{*, self};
    use crate::parser::parser;

    fn vm_from(input: &str) -> VM {
        let mut c = Compiler::new();
        c.compile_expr(parser::parse_from_expr(input).unwrap());
        VM::new(c.finish())
    }

    fn test_input(input: &str, result: &Object) {
        let mut vm = vm_from(input);
        vm.run();
        assert_eq!(vm.peek_top(), Some(result), "For input: {}", input);
    }

    #[test]
    fn op_const() {
        test_input("99", &Integer(99));
    }

    #[test]
    fn op_keyword_literals() {
        test_input("true", &True);
        test_input("false", &False);
        test_input("null", &Null);
    }

    #[test]
    fn equivalence() {
        test_input("true == true", &True);
        test_input("true == false", &False);
        test_input("true != true", &False);
        test_input("true != false", &True);

        test_input("3 == 3", &True);
        test_input("3 == 5", &False);
        test_input("3 != 3", &False);
        test_input("3 != 5", &True);
    }

    #[test]
    fn math_ops() {
        test_input("3 + 4", &Integer(7));
        test_input("3 - 4", &Integer(-1));
        test_input("3.0 * 4", &Float(12.0));
        test_input("4 / 2", &Float(2.0));
        test_input("4 div 2", &Integer(2));
        test_input("4 ** 2", &Integer(16));
        test_input("4 < 2", &False);
        test_input("4 <= 4", &True);
        test_input("4 > 2", &True);
        test_input("4 >= 2", &True);

        test_input("-(9)", &Integer(-9));
        test_input("-(1.0 * 2)", &Float(-2.0));

        test_input("!true", &False);
        test_input("!(false == true)", &True);
    }

    #[test]
    fn ternary() {
        test_input("if (1 >= 5) ? 1 + 1 : 2 * 2", &Integer(4));
        test_input("if (1 < 5) ? 1 + 1 : 2 * 2", &Integer(2));
    }
}
