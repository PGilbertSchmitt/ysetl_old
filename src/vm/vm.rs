use bytes::{Buf, Bytes};
use std::io::Cursor;

use crate::code::code::{self, OpCode, OpCodeU16};
use crate::compiler::compiler::Bytecode;
use crate::object::object::{Object, ObjectOps, BaseObject};
use crate::object::math::{math_op, ObjectMath};

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
    globals: Vec<Object>,
    match_stack: Vec<Object>,

    stack: Vec<Object>,
    last_pop: Object,
}

impl VM {
    pub fn new(bytecode: Bytecode) -> Self {
        VM {
            instructions: bytecode.instuctions,
            
            constants: bytecode.constants.into_iter().map(|bo| bo.wrap()).collect(),
            globals: Vec::with_capacity(bytecode.global_count),
            match_stack: Vec::new(),

            stack: Vec::with_capacity(STACK_SIZE),
            last_pop: BaseObject::Null.wrap(),
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
                code::Const::VAL => {
                    let ptr = c.get_u16();
                    let const_obj = self.constants[ptr as usize].reference();
                    self.stack.push(const_obj);
                },
                code::Null::VAL => self.stack.push(BaseObject::Null.wrap()),
                code::True::VAL => self.stack.push(BaseObject::True.wrap()),
                code::False::VAL => self.stack.push(BaseObject::False.wrap()),

                code::Pop::VAL => {
                    let popped = self.stack.pop().expect("Called pop on empty stack");
                    println!("Just popped: {:?}", popped);
                }

                code::SetGVar::VAL => {
                    let ptr = c.get_u16() as usize; 
                    // To do this in a straighforward manner, we would pop the stack, insert a reference
                    // to the globals vector, and push it back onto the stack, so we just leave it in the
                    // stack and reference it from there using `last` instead.
                    let top = self.stack.last().unwrap().reference();
                    self.globals.insert(ptr, top);
                }

                code::GetGVar::VAL => {
                    let ptr = c.get_u16() as usize;
                    let global = self.globals.get(ptr).unwrap().reference();
                    self.stack.push(global);
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

                code::Add::VAL
                | code::Subtract::VAL
                | code::Mult::VAL
                | code::Div::VAL
                | code::IntDiv::VAL
                | code::Exp::VAL
                | code::Lt::VAL
                | code::Lteq::VAL => {
                    let (right, left) = self.stack.pop_two();
                    let result = math_op(left.inner, right.inner, op).unwrap();
                    self.stack.push(result.wrap());
                }
                code::Eq::VAL => {
                    let (right, left) = self.stack.pop_two();
                    let result = if left == right {
                        BaseObject::True.wrap()
                    } else {
                        BaseObject::False.wrap()
                    };
                    self.stack.push(result);
                }
                code::Neq::VAL => {
                    let (right, left) = self.stack.pop_two();
                    self.stack.push(if left != right {
                        BaseObject::True.wrap()
                    } else {
                        BaseObject::False.wrap()
                    });
                }
                
                code::Negate::VAL => {
                    let val = self.stack.pop().unwrap();
                    self.stack.push(val.inner.negate().unwrap().wrap());
                }
                code::Not::VAL => {
                    let val = self.stack.pop().unwrap();
                    self.stack.push(val.not());
                }

                code => unimplemented!("Don't know how to execute code {code}"),
            }
        };

        self.last_pop.reference()
    }
}

// #[cfg(test)]
// mod tests {
//     use super::VM;
//     use crate::compiler::compiler::Compiler;
//     use crate::object::object::{ObjectOps, BaseObject::{self, *}};
//     use crate::parser::parser;

//     fn vm_from(input: &str) -> VM {
//         let mut c = Compiler::new();
//         c.compile_expr(parser::parse_from_expr(input).unwrap());
//         VM::new(c.finish())
//     }

//     fn test_input(input: &str, result: &BaseObject) {
//         let mut vm = vm_from(input);
//         vm.run();
//         assert!(vm.peek_top() == Some(&result.reference()), "For input: {}", input);
//     }

//     #[test]
//     fn op_const() {
//         test_input("99", &Integer(99));
//     }

//     #[test]
//     fn op_keyword_literals() {
//         test_input("true", &True);
//         test_input("false", &False);
//         test_input("null", &Null);
//     }

//     #[test]
//     fn equivalence() {
//         test_input("true == true", &True);
//         test_input("true == false", &False);
//         test_input("true != true", &False);
//         test_input("true != false", &True);

//         test_input("3 == 3", &True);
//         test_input("3 == 5", &False);
//         test_input("3 != 3", &False);
//         test_input("3 != 5", &True);
//     }

//     #[test]
//     fn math_ops() {
//         test_input("3 + 4", &Integer(7));
//         test_input("3 - 4", &Integer(-1));
//         test_input("3.0 * 4", &Float(12.0));
//         test_input("4 / 2", &Float(2.0));
//         test_input("4 div 2", &Integer(2));
//         test_input("4 ** 2", &Integer(16));
//         test_input("4 < 2", &False);
//         test_input("4 <= 4", &True);
//         test_input("4 > 2", &True);
//         test_input("4 >= 2", &True);

//         test_input("-(9)", &Integer(-9));
//         test_input("-(1.0 * 2)", &Float(-2.0));

//         test_input("!true", &False);
//         test_input("!(false == true)", &True);
//     }

//     #[test]
//     fn ternary() {
//         test_input("if (1 >= 5) ? 1 + 1 : 2 * 2", &Integer(4));
//         test_input("if (1 < 5) ? 1 + 1 : 2 * 2", &Integer(2));
//     }
// }
