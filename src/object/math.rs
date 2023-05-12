use super::object::Object::{self, *};
use crate::code::code::{codes, OpCode};

fn int_math(left: i64, right: i64, op: OpCode) -> Object {
    match op {
        codes::ADD => Integer(left + right),
        codes::SUBTRACT => Integer(left - right),
        codes::MULT => Integer(left * right),
        codes::DIV => Float(left as f64 / right as f64),
        codes::INT_DIV => {
            if right == 0 {
                panic!("Divide by zero error")
            };
            Integer(left / right)
        }
        codes::EXP => Integer(left.pow(right as u32)),
        codes::LT => if left < right { True } else { False },
        codes::LTEQ => if left <= right { True } else { False },
        _ => unimplemented!(),
    }
}

fn float_math(left: f64, right: f64, op: OpCode) -> Object {
    match op {
        codes::ADD => Float(left + right),
        codes::SUBTRACT => Float(left - right),
        codes::MULT => Float(left * right),
        codes::DIV => {
            if right == 0.0 {
                panic!("Divide by zero error");
            }
            Float(left / right)
        }
        codes::INT_DIV => {
            panic!("Operands for `div` must both be integers");
        }
        codes::EXP => Float(left.powf(right)),
        codes::LT => if left < right { True } else { False },
        codes::LTEQ => if left <= right { True } else { False },
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

    pub fn math(left: Object, right: Object, op: OpCode) -> Option<Object> {
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
