use std::rc::Rc;

use bytes::Bytes;

#[derive(Debug)]
pub struct Frame {
    ins: Rc<Bytes>,
    pub ptr: u64,
}

impl Frame {
    pub fn new(ins: Rc<Bytes>, ptr: u64) -> Self {
        Self {
            ins,
            ptr,
        }
    }

    pub fn instructions(&self) -> Rc<Bytes> {
        self.ins.clone()
    }
}
