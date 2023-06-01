use std::{fmt::Debug, rc::Rc};

use bytes::Bytes;

pub trait ObjectOps {
    fn not(&self) -> Self;
    fn truthy(&self) -> bool;
    fn is_int(&self) -> bool;   
    fn get_index(&self, index: &Object) -> Object;
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
    String(String),
    Tuple(Vec<Object>),
    Set(Vec<Object>),
    Function {
        ins: Rc<Bytes>,
        locals: usize,
        req_params: u16,
        opt_params: u16,
        locked_values: Vec<Object>
    },
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
            BaseObject::String(str) => str.len() > 0,
            BaseObject::Tuple(els) => els.len() > 0,
            BaseObject::Set(els) => els.len() > 0,
            BaseObject::Function {..} => true,
        }
    }

    fn is_int(&self) -> bool {
        match self {
            BaseObject::Integer(_) => true,
            _ => false,
        }
    }

    fn get_index(&self, index: &Object) -> Object {
        match self {
            Self::String(str) => {
                if let &&BaseObject::Integer(val) = &index.inner.as_ref() {
                    let char = str.chars().nth(val as usize).map_or_else(|| {
                        panic!("{} is out of index for string {}", val, str);
                    }, |ch| {
                        ch.to_string()
                    });
                    BaseObject::String(char).wrap()
                } else {
                    panic!("Cannot index into string with {:?}", index)
                }
            }
            Self::Tuple(elements) => {
                if let &&BaseObject::Integer(val) = &index.inner.as_ref() {
                    elements.get(val as usize).map_or_else(|| {
                        panic!("{} is out of index for vector", val);
                    }, |ch| {
                        ch.reference()
                    })
                } else {
                    panic!("Cannot index into string with {:?}", index)
                }
            }
            // Self::Set(elements) => {

            // }
            _ => unimplemented!()
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
            Self::String(str) => f.debug_tuple("str").field(str).finish(),
            Self::Tuple(els) => f.debug_tuple("tup").field(els).finish(),
            Self::Set(els) => f.debug_tuple("set").field(els).finish(),
            Self::Function {locked_values, ..} => f.debug_tuple("fn").field(locked_values).finish(),
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

    fn is_int(&self) -> bool {
        self.inner.is_int()
    }

    fn get_index(&self, index: &Object) -> Object {
        self.inner.get_index(index)
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
