use bytes::{Bytes, BytesMut};
use crate::object::object::Object as Obj;
use crate::parser::ast::{ExprST, BinOp};
use crate::code::code::{Op,codes};

pub struct Compiler {
    instructions: BytesMut,
    constants: Vec<Obj>,
}

pub struct BytecodeRef<'a> {
    pub instructions: &'a BytesMut,
    pub constants: &'a Vec<Obj>,
}

pub struct Bytecode {
    pub instuctions: Bytes,
    pub constants: Vec<Obj>,
}

impl Compiler {
    pub fn new() -> Self {
        Compiler { instructions: BytesMut::new(), constants: vec![] }
    }

    pub fn compile(&mut self, node: ExprST) {
        match node {
            ExprST::Integer(value) => {
                let const_ptr = self.add_const(Obj::Integer(value));
                self.emit(&codes::CONST.make_with(&[const_ptr]));
            },
            ExprST::Infix { op, left, right } => {
                self.compile(*left);
                self.compile(*right);
                self.emit_binop(op);
            },
            _ => unimplemented!()
        };
    }

    pub fn check(&self) -> BytecodeRef {
        BytecodeRef {
            instructions: &self.instructions,
            constants: &self.constants,
        }
    }

    pub fn finish(self) -> Bytecode {
        Bytecode {
            instuctions: self.instructions.freeze(),
            constants: self.constants
        }
    }

    fn emit(&mut self, bytes: &Bytes) {
        self.instructions.extend_from_slice(bytes);
    }

    fn emit_binop(&mut self, binop: BinOp) {
        let op = match binop {
            BinOp::Add => codes::ADD,
            BinOp::Subtract => codes::SUBTRACT,
            _ => unimplemented!(),
        };
        self.emit(&op.make());
    }

    // OPTIMIZE: Constants could be added to a hashmap keyed by constant's value, and
    // the value of the hashmap would be an incrementing index which is returned by the
    // `add_const` fn. Before passing constant list to VM, constant map would be converted
    // to a vector. However, I think this may result in few space saves since programs
    // with many many similar constants aren't common.
    fn add_const(&mut self, constant: Obj) -> usize {
        self.constants.push(constant);
        self.constants.len() - 1
    }
}