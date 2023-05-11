use std::fmt::Debug;

// This could be a little inefficient for space since some consts
// will be bigger than others, and enums are sized to fit the largest
// variant, but I prefer the speed gains from having the common data
// structures on the stack.
#[derive(Clone, Copy, PartialEq)]
pub enum Object {
    Null,
    True,
    False,
    Integer(i64),
    Float(f64),
}

impl Object {
    pub fn not(self) -> Object {
        match self {
            Object::True => Object::False,
            Object::False => Object::True,
            _ => panic!("NOT operation can only be used on boolean values"),
        }
    }
    
    pub fn truthy(self) -> bool {
        match self {
            Object::True => true,
            Object::False => false,
            Object::Null => false,
            Object::Integer(val) => val != 0,
            Object::Float(val) => val != 0.0,
        }
    }
}

impl Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Null => f.write_str("null"),
            Self::True => f.write_str("true"),
            Self::False => f.write_str("false"),
            Self::Integer(val) => f.debug_tuple("int").field(val).finish(),
            Self::Float(val) => f.debug_tuple("float").field(val).finish(),
        }
    }
}
