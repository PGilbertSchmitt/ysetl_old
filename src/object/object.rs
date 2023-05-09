use std::fmt::Debug;

// This could be a little inefficient for space since some consts
// will be bigger than others, and enums are sized to fit the largest
// variant, but I prefer the speed gains from having the common data
// structures on the stack.
#[derive(Clone, Copy)]
pub enum Object {
    Null,
    Integer(i64),
    Float(f64),
}

impl Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Null => f.debug_tuple("null").finish(),
            Self::Integer(val) => f.debug_tuple("int").field(val).finish(),
            Self::Float(val) => f.debug_tuple("float").field(val).finish(),
        }
    }
}
