use std::rc::Rc;

use bytes::Bytes;

#[derive(Debug)]
pub struct Frame {
    ins: Rc<Bytes>,
    pub ins_ptr: u64,
    pub stack_ptr: usize,
}

impl Frame {
    pub fn new(ins: Rc<Bytes>, ins_ptr: u64, stack_ptr: usize) -> Self {
        Self {
            ins,
            ins_ptr,
            stack_ptr,
        }
    }

    pub fn instructions(&self) -> Rc<Bytes> {
        self.ins.clone()
    }
}
