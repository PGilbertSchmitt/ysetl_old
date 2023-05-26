use std::rc::Rc;

use super::object::BaseObject::{self, *};
use crate::code::code::{self, OpCode};

pub trait ObjectMath {
    fn to_float(&self) -> Option<Self> where Self: Sized;
    fn negate(&self) -> Option<Self> where Self: Sized;
}

fn int_math(left: i64, right: i64, op: u8) -> BaseObject {
    match op {
        code::Add::VAL => Integer(left + right),
        code::Subtract::VAL => Integer(left - right),
        code::Mult::VAL => Integer(left * right),
        code::Div::VAL => Float(left as f64 / right as f64),
        code::IntDiv::VAL => {
            if right == 0 {
                panic!("Divide by zero error")
            };
            Integer(left / right)
        }
        code::Exp::VAL => Integer(left.pow(right as u32)),
        code::Lt::VAL => if left < right { True } else { False },
        code::Lteq::VAL => if left <= right { True } else { False },
        _ => unimplemented!(),
    }
}

fn float_math(left: f64, right: f64, op: u8) -> BaseObject {
    match op {
        code::Add::VAL => Float(left + right),
        code::Subtract::VAL => Float(left - right),
        code::Mult::VAL => Float(left * right),
        code::Div::VAL => {
            if right == 0.0 {
                panic!("Divide by zero error");
            }
            Float(left / right)
        }
        code::IntDiv::VAL => {
            panic!("Operands for `div` must both be integers");
        }
        code::Exp::VAL => Float(left.powf(right)),
        code::Lt::VAL => if left < right { True } else { False },
        code::Lteq::VAL => if left <= right { True } else { False },
        _ => unimplemented!(),
    }
}

impl ObjectMath for BaseObject {
    fn to_float(&self) -> Option<BaseObject> {
        match self {
            Float(val) => Some(Float(*val)),
            Integer(val) => Some(Float(*val as f64)),
            _ => None,
        }
    }

    fn negate(&self) -> Option<BaseObject> {
        match self {
            Integer(value) => Some(Integer(-value)),
            Float(value) => Some(Float(-value)),
            other => panic!("Cannot negate {:?}", other),
        }
    }
}

pub fn math_op(left: &Rc<BaseObject>, right: &Rc<BaseObject>, op: u8) -> Option<BaseObject> {
    match (left.as_ref(), right.as_ref()) {
        (&Integer(left), &Integer(right)) => {
            return Some(int_math(left, right, op));
        }
        (&Integer(_), &Float(_)) | (&Float(_), &Integer(_)) | (&Float(_), &Float(_)) => {
            let (Some(Float(left_val)), Some(Float(right_val))) = (left.to_float(), right.to_float()) else {
                return None
            };
            return Some(float_math(left_val, right_val, op));
        }
        _ => None,
    }
}
