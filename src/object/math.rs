use super::object::Object::{self, *};
use crate::code::code::{OpCode,Op,codes};

fn int_math(left: i64, right: i64, op: OpCode) -> i64 {
    match op {
        codes::ADD => left + right,
        codes::SUBTRACT => left - right,
        _ => unimplemented!(),
    }
}

fn float_math(left: f64, right: f64, op: OpCode) -> f64 {
    match op {
        codes::ADD => left + right,
        codes::SUBTRACT => left - right,
        _ => unimplemented!(),
    }
}

impl Object {
    fn to_float(self) -> Result<Object, String> {
        match self {
            Float(_) => Ok(self),
            Integer(val) => Ok(Float(val as f64)),
            _ => panic!("Cannot convert type {:?} to float", self),
        }
    }

    pub fn math(left: Object, right: Object, op: OpCode) -> Result<Object, String> {
        if let (Integer(left), Integer(right)) = (left, right) {
            return Ok(Integer(int_math(left, right, op)));
        }

        if let (Float(left), Float(right)) = (left.to_float()?, right.to_float()?) {
            return Ok(Float(float_math(left, right, op)));
        }

        Err(format!("Could not perform {:?} with {:?} and {:?}", op.lookup().1, left, right))
    }
}