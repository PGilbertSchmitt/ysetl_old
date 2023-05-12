use crate::code::code::{codes, Op, OpCode};
use crate::object::object::Object as Obj;
use crate::parser::ast::{BinOp, Case, ExprST, PreOp, Program};
use bytes::{BufMut, Bytes, BytesMut};

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
        self.compile_expr_list(node.expressions);
    }

    pub fn compile_expr_list(&mut self, exprs: Vec<ExprST>) {
        for expr in exprs.into_iter() {
            self.compile_expr(expr);
            // OPTIMIZE: If the last op after the above line runs is something that would only
            // push a value onto the stack, just remove it and omit the following pop.
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
            ExprST::Infix { op, mut left, mut right } => {
                // Need special jump logic when op is AND/OR/IMPL so that right side is only
                // evaluated in correct circumstances.
                if let BinOp::GT | BinOp::GTEQ = op { (left, right) = (right, left) }
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
            ExprST::Ternary {
                condition,
                consequence,
                alternative,
            } => {
                self.compile_expr(*condition);

                let jnt_operand_ptr = self.instructions.len() + 1;
                self.emit(&codes::JUMP_NOT_TRUE.make_with(&[usize::MAX]));
                self.compile_expr(*consequence);
                let jump_operand_ptr = self.instructions.len() + 1;
                self.emit(&codes::JUMP.make_with(&[usize::MAX]));
                let jnt_location = self.cur_ip();
                self.compile_expr(*alternative);

                self.overwrite_u16(jnt_operand_ptr, jnt_location);
                self.overwrite_u16(jump_operand_ptr, self.cur_ip());
            }
            ExprST::Switch { input, cases } => match input {
                Some(expr) => self.compile_match_switch(*expr, cases),
                None => self.compile_bool_switch(cases),
            },
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

    fn cur_ip(&self) -> u16 {
        self.instructions.len() as u16
    }

    fn emit(&mut self, bytes: &Bytes) {
        self.instructions.extend_from_slice(bytes);
    }

    fn emit_const(&mut self, const_ptr: usize) {
        self.instructions
            .extend_from_slice(&codes::CONST.make_with(&[const_ptr]));
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
            BinOp::GT => codes::LT,
            BinOp::GTEQ => codes::LTEQ,
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

    fn overwrite(&mut self, at: usize, value: Bytes) {
        for (i, byte) in value.into_iter().enumerate() {
            self.instructions[at + i] = byte;
        }
    }

    fn overwrite_u16(&mut self, at: usize, value: u16) {
        let mut bytes = BytesMut::with_capacity(2);
        bytes.put_u16(value);
        self.overwrite(at, bytes.freeze())
    }

    fn compile_match_switch(&mut self, input: ExprST, cases: Vec<Case>) {
        self.compile_expr(input);
        self.emit(&codes::PUSH_MATCH.make());
        self.compile_switch_cases(cases, codes::JUMP_NOT_MATCH);
        self.emit(&codes::POP_MATCH.make());
    }

    fn compile_bool_switch(&mut self, cases: Vec<Case>) {
        self.compile_switch_cases(cases, codes::JUMP_NOT_TRUE);
    }

    fn compile_switch_cases(&mut self, cases: Vec<Case>, cond_jump_op: OpCode) {
        let mut jmp_operand_ptrs: Vec<usize> = vec![];
        let mut last_cond_jump_operand_ptr: Option<usize> = None;
        
        for Case {
            condition,
            consequence,
            null_return,
        } in cases.into_iter() {
            if let Some(ptr) = last_cond_jump_operand_ptr {
                self.overwrite_u16(ptr, self.cur_ip());
            }

            // Tilde case causes condition to be None, so we don't add a JUMP_NOT_TRUE
            let default_case = condition.is_none();
            condition.map(|expr| self.compile_expr(*expr));

            if !default_case {
                last_cond_jump_operand_ptr = Some(self.instructions.len() + 1);
                self.emit(&cond_jump_op.make_with(&[usize::MAX]));
                self.compile_expr_list(consequence);
                self.handle_null_return(null_return);
                jmp_operand_ptrs.push(self.instructions.len() + 1);
                self.emit(&codes::JUMP.make_with(&[usize::MAX]));
            } else {
                self.compile_expr_list(consequence);
                self.handle_null_return(null_return);
                // Any cases that follow the default case will not be compiled because they're unreachable
                break;
            }
        }

        let cur_pos = self.cur_ip();
        for jmp_operand_ptr in jmp_operand_ptrs {
            self.overwrite_u16(jmp_operand_ptr, cur_pos);
        }
    }

    fn handle_null_return(&mut self, null_return: bool) {
        if null_return {
            self.emit(&codes::NULL.make());
        } else {
            self.instructions.truncate(self.instructions.len() - 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Bytecode, Compiler};
    use crate::code::code::codes::*;
    use crate::code::debug::print_bytes;
    use crate::object::object::Object::*;
    use crate::parser::parser;
    use bytes::Bytes;

    fn compile(input: &str) -> Bytecode {
        let mut c = Compiler::new();
        c.compile_expr(parser::parse_from_expr(input).unwrap());
        c.finish()
    }

    fn compile_program(input: &str) -> Bytecode {
        let wrapped_input = format!("program :any; {}", input);
        let mut c = Compiler::new();
        c.compile_program(parser::parse_from_program(&wrapped_input).unwrap());
        c.finish()
    }

    fn assert_bytes(result: &Bytecode, bytes: Vec<u8>) {
        let expected = &Bytes::from(bytes);
        let actual = &result.instuctions;
        let equal = expected == actual;
        if !equal {
            panic!(
                "\n\nExpected:\n{}\n\nInstead, got:\n{}\n\n",
                print_bytes(&expected),
                print_bytes(&actual)
            )
        }
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
        assert_eq!(
            compile("3 + 4").instuctions,
            Bytes::from(vec![CONST, 0, 0, CONST, 0, 1, ADD])
        );
        assert_eq!(
            compile("3 - 4").instuctions,
            Bytes::from(vec![CONST, 0, 0, CONST, 0, 1, SUBTRACT])
        );
        assert_eq!(
            compile("3 + (4 / 5)").instuctions,
            Bytes::from(vec![CONST, 0, 0, CONST, 0, 1, CONST, 0, 2, DIV, ADD])
        );
    }

    #[test] #[rustfmt::skip]
    fn ternary() {
        // let ternary_code = compile_program("if true ? 1 : 2; 99;");
        // assert_eq!(ternary_code.instuctions, Bytes::from(vec![
        assert_bytes(&compile_program("if true ? 1 : 2; 99;"), vec![
            // 0
            TRUE,
            // 1
            JUMP_NOT_TRUE, 0, 10,
            // 4
            CONST, 0, 0,
            // 7
            JUMP, 0, 13,
            // 10
            CONST, 0, 1,
            // 13
            POP,
            // 14
            CONST, 0, 2,
            // 17
            POP,
            // 18
        ]);
    }
}
