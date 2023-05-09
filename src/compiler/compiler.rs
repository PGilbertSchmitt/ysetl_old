use crate::code::code::{codes, Op};
use crate::object::object::Object as Obj;
use crate::parser::ast::{BinOp, ExprST};
use bytes::{Bytes, BytesMut};

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
        Compiler {
            instructions: BytesMut::new(),
            constants: vec![],
        }
    }

    pub fn compile(&mut self, node: ExprST) {
        match node {
            ExprST::Null => {
                self.emit(&codes::NULL.make());
            }
            ExprST::True => {
                self.emit(&codes::TRUE.make());
            }
            ExprST::False => {
                self.emit(&codes::FALSE.make());
            }
            ExprST::Integer(value) => {
                let const_ptr = self.add_const(Obj::Integer(value));
                self.emit(&codes::CONST.make_with(&[const_ptr]));
            }
            ExprST::Infix { op, left, right } => {
                // Need special jump logic when op is AND/OR/IMPL so that right side is only
                // evaluated in correct circumstances.
                self.compile(*left);
                self.compile(*right);
                self.emit_binop(op);
            }
            _ => unimplemented!(),
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
            constants: self.constants,
        }
    }

    fn emit(&mut self, bytes: &Bytes) {
        self.instructions.extend_from_slice(bytes);
    }

    fn emit_binop(&mut self, binop: BinOp) {
        let op = match binop {
            BinOp::NullCoal => codes::NULL_COAL,
            BinOp::TupleStart => codes::TUPLE_START,
            BinOp::Exp => codes::TUPLE_START,
            BinOp::Mult => codes::MULT,
            BinOp::Inter => codes::INTER,
            BinOp::Div => codes::DIV,
            BinOp::Mod => codes::MOD,
            BinOp::IntDiv => codes::INT_DIV,
            BinOp::Add => codes::ADD,
            BinOp::Subtract => codes::SUBTRACT,
            BinOp::With => codes::WITH,
            BinOp::Less => codes::LESS,
            BinOp::Union => codes::UNION,
            BinOp::In => codes::IN,
            BinOp::Notin => codes::NOTIN,
            BinOp::Subset => codes::SUBSET,
            BinOp::LT => codes::LT,
            BinOp::LTEQ => codes::LTEQ,
            BinOp::GT => codes::GT,
            BinOp::GTEQ => codes::GTEQ,
            BinOp::EQ => codes::EQ,
            BinOp::NEQ => codes::NEQ,
            BinOp::And => codes::AND,
            BinOp::Or => codes::OR,
            BinOp::Impl => codes::IMPL,
            BinOp::Iff => codes::IFF,
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
