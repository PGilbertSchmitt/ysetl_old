use std::{fmt::Debug, rc::Rc};

pub trait ObjectOps {
    fn not(&self) -> Self;
    fn truthy(&self) -> bool;
}

// This could be a little inefficient for space since some consts
// will be bigger than others, and enums are sized to fit the largest
// variant, but I prefer the speed gains from having the common data
// structures on the stack.
#[derive(PartialEq)]
pub enum BaseObject {
    Null,
    True,
    False,
    Integer(i64),
    Float(f64),
}

impl BaseObject {
    /** Wrap a BaseObject instance with an Object. */
    pub fn wrap(self) -> Object {
        Object { inner: Rc::new(self) }
    }
}

impl ObjectOps for BaseObject {
    fn not(&self) -> BaseObject {
        match self {
            BaseObject::True => BaseObject::False,
            BaseObject::False => BaseObject::True,
            _ => panic!("NOT operation can only be used on boolean values"),
        }
    }

    fn truthy(&self) -> bool {
        match self {
            BaseObject::True => true,
            BaseObject::False => false,
            BaseObject::Null => false,
            BaseObject::Integer(val) => *val != 0,
            BaseObject::Float(val) => *val != 0.0,
        }
    }
}

impl Debug for BaseObject {
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

pub struct Object {
    pub inner: Rc<BaseObject>
}

impl Object {

    /** Create a new reference to the same inner base object */
    pub fn reference(&self) -> Object {
        Object { inner: self.inner.clone() }
    }
}

impl ObjectOps for Object {
    fn not(&self) -> Self {
        self.inner.not().wrap()
    }

    fn truthy(&self) -> bool {
        self.inner.truthy()
    }
}

impl Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
