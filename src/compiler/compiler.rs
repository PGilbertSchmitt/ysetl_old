use crate::code::code::{codes, Op};
use crate::object::object::Object as Obj;
use crate::parser::ast::{BinOp, ExprST, Program, PreOp};
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

    pub fn compile_program(&mut self, node: Program) {
        for expr in node.expressions.into_iter() {
            self.compile_expr(expr);
            self.emit(&codes::POP.make());
        }
    }

    pub fn compile_expr(&mut self, node: ExprST) {
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
                self.emit_const(const_ptr);
            }
            ExprST::Float(value) => {
                let const_ptr = self.add_const(Obj::Float(value));
                self.emit_const(const_ptr);
            }
            ExprST::Infix { op, left, right } => {
                // Need special jump logic when op is AND/OR/IMPL so that right side is only
                // evaluated in correct circumstances.
                self.compile_expr(*left);
                self.compile_expr(*right);
                self.emit_binop(op);
            }
            ExprST::Prefix { op, right } => {
                let right = *right;
                // couple of small easy optimizations (since I couldn't figure out how to
                // smoothly get my parser to do this without conflicting with PreOp::Negate)
                if let (&PreOp::Negate, &ExprST::Integer(value)) = (&op, &right) {
                    let const_ptr = self.add_const(Obj::Integer(-value));
                    self.emit_const(const_ptr);
                } else if let (&PreOp::Negate, &ExprST::Float(value)) = (&op, &right) {
                    let const_ptr = self.add_const(Obj::Float(-value));
                    self.emit_const(const_ptr);
                } else {
                    self.compile_expr(right);
                    self.emit_preop(op);
                }
            }
            node => unimplemented!("Not sure how to compile {:?}", node),
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

    fn emit_const(&mut self, const_ptr: usize) {
        self.instructions.extend_from_slice(&codes::CONST.make_with(&[const_ptr]));
    }

    fn emit_binop(&mut self, binop: BinOp) {
        let op = match binop {
            BinOp::NullCoal => codes::NULL_COAL,
            BinOp::TupleStart => codes::TUPLE_START,
            BinOp::Exp => codes::EXP,
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

    fn emit_preop(&mut self, preop: PreOp) {
        match preop {
            PreOp::Id => {} // No op, though this may change
            PreOp::Negate => self.emit(&codes::NEGATE.make()),
            PreOp::DynVar => self.emit(&codes::DYN_VAR.make()),
            PreOp::Size => self.emit(&codes::SIZE.make()),
            PreOp::Not => self.emit(&codes::NOT.make()),
        }
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

#[cfg(test)]
mod tests {
    use bytes::{Bytes};
    use super::{Compiler, Bytecode};
    use crate::parser::parser;
    use crate::code::code::codes::*;
    use crate::object::object::Object::*;

    fn compile(input: &str) -> Bytecode {
        let mut c = Compiler::new();
        c.compile_expr(parser::parse_from_expr(input).unwrap());
        c.finish()
    }

    #[test]
    fn literals() {
        let int_code = compile("3");
        assert_eq!(int_code.instuctions, Bytes::from(vec![CONST, 0, 0]));
        assert_eq!(int_code.constants[0], Integer(3));

        let float_code = compile("3.0");
        assert_eq!(float_code.instuctions, Bytes::from(vec![CONST, 0, 0]));
        assert_eq!(float_code.constants[0], Float(3.0));

        let true_code = compile("true");
        assert_eq!(true_code.instuctions, Bytes::from(vec![TRUE]));
        assert!(true_code.constants.is_empty());

        let false_code = compile("false");
        assert_eq!(false_code.instuctions, Bytes::from(vec![FALSE]));
        assert!(false_code.constants.is_empty());

        let null_code = compile("null");
        assert_eq!(null_code.instuctions, Bytes::from(vec![NULL]));
        assert!(null_code.constants.is_empty());
        
        let negative_int = compile("-1");
        assert_eq!(negative_int.instuctions, Bytes::from(vec![CONST, 0, 0]));
        assert_eq!(negative_int.constants[0], Integer(-1));
        
        let negative_float = compile("-1.0");
        assert_eq!(negative_float.instuctions, Bytes::from(vec![CONST, 0, 0]));
        assert_eq!(negative_float.constants[0], Float(-1.0));
    }

    #[test]
    fn simple_math() {
        assert_eq!(compile("3 + 4").instuctions, Bytes::from(vec![CONST, 0, 0, CONST, 0, 1, ADD]));
        assert_eq!(compile("3 - 4").instuctions, Bytes::from(vec![CONST, 0, 0, CONST, 0, 1, SUBTRACT]));
        assert_eq!(compile("3 + (4 / 5)").instuctions, Bytes::from(vec![CONST, 0, 0, CONST, 0, 1, CONST, 0, 2, DIV, ADD]));
    }
}
