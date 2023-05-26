use bytes::{Buf};
use std::io::Cursor;
use std::rc::Rc;

use crate::code::code::{self, OpCode};
use crate::code::debug::lookup;
use crate::compiler::compiler::Bytecode;
use crate::object::math::{math_op, ObjectMath};
use crate::object::object::{BaseObject, Object, ObjectOps};

use super::frame::Frame;

const STACK_SIZE: usize = 2048;
const MAX_FRAMES: usize = 2048;

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
    call_stack: Vec<Frame>,
    constants: Vec<Object>,
    globals: Vec<Option<Object>>,
    match_stack: Vec<Object>,

    stack: Vec<Object>,
}

impl VM {
    pub fn new(bytecode: Bytecode) -> Self {
        let main_frame = Frame::new(Rc::new(bytecode.instuctions), 0);
        let constants = bytecode.constants.into_iter().map(|bo| bo.wrap()).collect();
        // Must be initialized so that insertions can happen in any order
        let globals = (0..bytecode.global_count).into_iter().map(|_| None).collect();

        VM {
            call_stack: Vec::from([main_frame]),
            constants,
            globals,
            match_stack: Vec::new(),

            stack: Vec::with_capacity(STACK_SIZE),
        }
    }

    pub fn peek_top(&self) -> Option<&Object> {
        self.stack.last()
    }

    pub fn run(&mut self) -> Object {
        // Duplicate Rc for the current instructions, used to keep reference for cursor during function calls
        let mut cur_ins = self.cur_frame().instructions();
        let mut c = Cursor::new(cur_ins.as_ref());
        let mut last_pop: Option<Object> = None;

        while c.has_remaining() {
            let op = c.get_u8();
            match op {
                code::Const::VAL => {
                    let ptr = c.get_u16();
                    let const_obj = self.constants[ptr as usize].reference();
                    self.stack.push(const_obj);
                }
                code::Null::VAL => self.stack.push(BaseObject::Null.wrap()),
                code::True::VAL => self.stack.push(BaseObject::True.wrap()),
                code::False::VAL => self.stack.push(BaseObject::False.wrap()),

                code::Pop::VAL => {
                    last_pop = self.stack.pop();
                    if last_pop.is_none() {
                        panic!("Called pop on empty stack");
                    }
                }

                code::SetGVar::VAL => {
                    let ptr = c.get_u16() as usize;
                    // To do this in a straighforward manner, we would pop the stack, insert a reference
                    // to the globals vector, and push it back onto the stack, so we just leave it in the
                    // stack and reference it from there using `last` instead.
                    let top = self.stack.last().unwrap().reference();
                    self.globals[ptr] = Some(top);
                }

                code::GetGVar::VAL => {
                    let ptr = c.get_u16() as usize;
                    if let Some(global) = self.globals.get(ptr).unwrap() {
                        self.stack.push(global.reference());
                    } else {
                        panic!("Variable hasn't been initialized");
                    };
                }

                code::Tuple::VAL => {
                    let size = c.get_u16() as usize;
                    let drain_start: usize = self.stack.len() - size;
                    let elements: Vec<Object> = self.stack.drain(drain_start..).collect();
                    self.stack.push(BaseObject::Tuple(elements).wrap());
                }

                code::Set::VAL => {
                    let size = c.get_u16() as usize;
                    let drain_start: usize = self.stack.len() - size;
                    let elements: Vec<Object> = self.stack.drain(drain_start..).collect();
                    self.stack.push(BaseObject::Set(elements).wrap());
                }

                code::TupleRn::VAL => {
                    let size = c.get_u16();
                    let elements = self.calculate_range(size);
                    self.stack.push(BaseObject::Tuple(elements).wrap());
                }

                code::SetRn::VAL => {
                    let size = c.get_u16();
                    let elements = self.calculate_range(size);
                    self.stack.push(BaseObject::Set(elements).wrap());
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

                code::Index::VAL => {
                    let index = self.stack.pop().unwrap();
                    let target = self.stack.pop().unwrap();
                    self.stack.push(target.get_index(&index))
                }

                code::Call::VAL => {
                    let object = self.stack.pop().unwrap();
                    match &object.inner.as_ref() {
                        &BaseObject::Function(bytes) => {
                            let new_frame = Frame::new(bytes.clone(), c.position());
                            cur_ins = bytes.clone();
                            c = Cursor::new(&cur_ins.as_ref());
                            self.push_frame(new_frame);
                        }
                        other => panic!("Cannot call {:?}", other)
                    }
                }

                code::Return::VAL => {
                    let last_frame = self.pop_frame();
                    cur_ins = self.cur_frame().instructions();
                    c = Cursor::new(&cur_ins.as_ref());
                    c.set_position(last_frame.ptr);
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
                    let result = math_op(&left.inner, &right.inner, op);
                    // There's probably a better way to do all this
                    if result.is_none() {
                        panic!("Could not perform {} on types {:?} and {:?}", lookup(op).unwrap().1, left.inner, right.inner)
                    }
                    self.stack.push(result.unwrap().wrap());
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
        }

        last_pop.unwrap().reference()
    }

    fn cur_frame(&self) -> &Frame {
        self.call_stack.last().expect("No frames found, this shouldn't be possible")
    }

    fn push_frame(&mut self, f: Frame) {
        if self.call_stack.len() >= MAX_FRAMES {
            panic!("Stack overflow!");
        }
        self.call_stack.push(f);
    }

    fn pop_frame(&mut self) -> Frame {
        self.call_stack.pop().expect("Cannot pop empty callstack")
    }

    fn calculate_range(&mut self, size: u16) -> Vec<Object> {
        let start = self.stack.pop().unwrap();
        let end = self.stack.pop().unwrap();
        let step_opt = match size {
            2 => None,
            // Unwrapping then rewrapping looks dumb, but the pop must fail if it's None
            3 => Some(self.stack.pop().unwrap()),
            _ => unreachable!(),
        };
        if let (&BaseObject::Integer(start), &BaseObject::Integer(end)) =
            (start.inner.as_ref(), end.inner.as_ref())
        {
            let step = step_opt.map_or(1, |v| {
                if let &BaseObject::Integer(v) = v.inner.as_ref() {
                    v - start
                } else {
                    panic!("Range elements must evaluate to integers");
                }
            });

            // These would all iterate forever. Like in the original ISetL, this instead evaluates
            // to an empty range. Initially, I would have prefered this to fail, but I can see it
            // being handy to check if a range is valid if the resulting collection is truthy.
            if step == 0 || (step > 0 && start > end) || (step < 0 && start < end) {
                return vec![];
            }

            let mut values: Vec<Object> = Vec::new();
            let mut x = start;
            loop {
                if (step > 0 && x > end) || (step < 0 && x < end) {
                    break;
                }
                values.push(BaseObject::Integer(x).wrap());
                x += step;
            }

            values
        } else {
            panic!("Range elements must evaluate to integers");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::VM;
    use crate::compiler::compiler::Compiler;
    use crate::object::object::BaseObject::{self, *};
    use crate::parser::parser;

    fn vm_from(input: &str) -> VM {
        let mut c = Compiler::new();
        c.compile_expr(parser::parse_from_expr(input).unwrap());
        VM::new(c.finish())
    }

    fn test_input(input: &str, result: BaseObject) {
        let mut vm = vm_from(input);
        vm.run();
        assert!(
            vm.peek_top() == Some(&result.wrap()),
            "For input: {}",
            input
        );
    }

    #[test]
    fn op_const() {
        test_input("99", Integer(99));
    }

    #[test]
    fn op_keyword_literals() {
        test_input("true", True);
        test_input("false", False);
        test_input("null", Null);
    }

    #[test]
    fn equivalence() {
        test_input("true == true", True);
        test_input("true == false", False);
        test_input("true != true", False);
        test_input("true != false", True);

        test_input("3 == 3", True);
        test_input("3 == 5", False);
        test_input("3 != 3", False);
        test_input("3 != 5", True);
    }

    #[test]
    fn math_ops() {
        test_input("3 + 4", Integer(7));
        test_input("3 - 4", Integer(-1));
        test_input("3.0 * 4", Float(12.0));
        test_input("4 / 2", Float(2.0));
        test_input("4 div 2", Integer(2));
        test_input("4 ** 2", Integer(16));
        test_input("4 < 2", False);
        test_input("4 <= 4", True);
        test_input("4 > 2", True);
        test_input("4 >= 2", True);

        test_input("-(9)", Integer(-9));
        test_input("-(1.0 * 2)", Float(-2.0));

        test_input("!true", False);
        test_input("!(false == true)", True);
    }

    #[test]
    fn ternary() {
        test_input("if (1 >= 5) ? 1 + 1 : 2 * 2", Integer(4));
        test_input("if (1 < 5) ? 1 + 1 : 2 * 2", Integer(2));
    }
}
