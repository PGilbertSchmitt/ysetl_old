use super::object::Object::{self, *};
use crate::code::code::{self, OpCode};

fn int_math(left: i64, right: i64, op: u8) -> Object {
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

fn float_math(left: f64, right: f64, op: u8) -> Object {
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

impl Object {
    fn to_float(&self) -> Option<Object> {
        match self {
            Float(_) => Some(*self),
            Integer(val) => Some(Float(*val as f64)),
            _ => None,
        }
    }

    pub fn negate(self) -> Object {
        match self {
            Integer(value) => Integer(-value),
            Float(value) => Float(-value),
            other => panic!("Cannot negate {:?}", other),
        }
    }

    pub fn math(left: Object, right: Object, op: u8) -> Option<Object> {
        match (left, right) {
            (Integer(left), Integer(right)) => {
                return Some(int_math(left, right, op));
            }
            (Integer(_), Float(_)) | (Float(_), Integer(_)) | (Float(_), Float(_)) => {
                let (Some(Float(left_val)), Some(Float(right_val))) = (left.to_float(), right.to_float()) else {
                    return None
                };
                return Some(float_math(left_val, right_val, op));
            }
            _ => None,
        }
    }
}
