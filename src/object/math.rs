use super::object::Object::{self, *};
use crate::code::code::{codes, OpCode};

fn int_math(left: i64, right: i64, op: OpCode) -> i64 {
    match op {
        codes::ADD => left + right,
        codes::SUBTRACT => left - right,
        codes::MULT => left * right,
        codes::INT_DIV => {
            if right == 0 {
                panic!("Divide by zero error")
            };
            left / right
        }
        codes::EXP => left.pow(right as u32),
        _ => unimplemented!(),
    }
}

fn float_math(left: f64, right: f64, op: OpCode) -> f64 {
    match op {
        codes::ADD => left + right,
        codes::SUBTRACT => left - right,
        codes::MULT => left * right,
        codes::DIV => {
            if right == 0.0 {
                panic!("Divide by zero error");
            }
            left / right
        }
        codes::INT_DIV => {
            panic!("Operands for `div` must both be integers");
        }
        codes::EXP => left.powf(right),
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

    pub fn math(left: Object, right: Object, op: OpCode) -> Option<Object> {
        match (op, left, right) {
            (codes::DIV, Integer(left), Integer(right)) => {
                return Some(Float(float_math(left as f64, right as f64, codes::DIV)))
            }
            (op, Integer(left), Integer(right)) => {
                return Some(Integer(int_math(left, right, op)));
            }
            (op, Integer(_), Float(_)) | (op, Float(_), Integer(_)) | (op, Float(_), Float(_)) => {
                let (Some(Float(left_val)), Some(Float(right_val))) = (left.to_float(), right.to_float()) else {
                    return None
                };
                return Some(Float(float_math(left_val, right_val, op)));
            }
            _ => None,
        }
    }
}
