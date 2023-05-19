use crate::code::code::{self, OpCodeMake, OpCodeMakeWithU16};
use crate::object::object::{BaseObject};
use crate::parser::ast::{BinOp, Case, ExprST, PreOp, Program, LHS, Former, Postfix};
use bytes::{BufMut, Bytes, BytesMut};

use super::symbols::SymbolMap;

pub struct Compiler {
    constants: Vec<BaseObject>,    
    symbol_map: SymbolMap,

    scopes: Vec<ScopeCtx>,
    scope_ptr: usize,
}

pub struct BytecodeRef<'a> {
    pub instructions: &'a BytesMut,
    pub constants: &'a Vec<BaseObject>,
    pub global_count: usize,
}

pub struct Bytecode {
    pub instuctions: Bytes,
    pub constants: Vec<BaseObject>,
    pub global_count: usize,
}

struct ScopeCtx {
    pub instructions: BytesMut,
}

impl Compiler {
    pub fn new() -> Self {
        let global_scope = ScopeCtx { instructions: BytesMut::new() };

        Compiler {
            constants: vec![],
            symbol_map: SymbolMap::new(),

            scopes: vec![global_scope],
            scope_ptr: 0,
        }
    }

    fn current_instructions(&self) -> &BytesMut {
        let top_scope = self.scopes.last().expect("Can't find any scopes");
        &top_scope.instructions
    }

    fn ins_len(&self) -> usize {
        self.current_instructions().len()
    }

    pub fn compile_program(&mut self, node: Program) {
        self.compile_expr_list(node.expressions, true);
    }

    pub fn compile_expr_list(&mut self, exprs: Vec<ExprST>, with_pop: bool) {
        for expr in exprs.into_iter() {
            self.compile_expr(expr);
            // OPTIMIZE: If the last op after the above line runs is something that would only
            // push a value onto the stack, just remove it and omit the following pop.
            if with_pop { self.emit(&code::Pop.make()); }
        }
    }

    pub fn compile_expr(&mut self, node: ExprST) {
        match node {
            ExprST::Null => {
                self.emit(&code::Null.make());
            }
            ExprST::True => {
                self.emit(&code::True.make());
            }
            ExprST::False => {
                self.emit(&code::False.make());
            }
            ExprST::Integer(value) => {
                let const_ptr = self.add_const(BaseObject::Integer(value));
                self.emit_const(const_ptr);
            }
            ExprST::Float(value) => {
                let const_ptr = self.add_const(BaseObject::Float(value));
                self.emit_const(const_ptr);
            }
            ExprST::Ident(name) => {
                let i = self.symbol_map.lookup(name).expect("'{}' is undefined in current scope").index;
                self.emit(&code::GetGVar.make(i));
            }
            ExprST::String(value) => {
                let const_ptr = self.add_const(BaseObject::String(value.to_owned()));
                self.emit_const(const_ptr);
            }
            ExprST::TupleLiteral(former) => {
                self.compile_former(former, code::Tuple, code::TupleRn);
            }
            ExprST::SetLiteral(former) => {
                self.compile_former(former, code::Set, code::SetRn);
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
                    let const_ptr = self.add_const(BaseObject::Integer(-value));
                    self.emit_const(const_ptr);
                } else if let (&PreOp::Negate, &ExprST::Float(value)) = (&op, &right) {
                    let const_ptr = self.add_const(BaseObject::Float(-value));
                    self.emit_const(const_ptr);
                } else {
                    self.compile_expr(right);
                    self.emit_preop(op);
                }
            }
            ExprST::Postfix { left, selector } => {
                self.compile_expr(*left);
                match selector {
                    Postfix::Index(index) => {
                        self.compile_expr(*index);
                        self.emit(&code::Index.make());
                    }
                    _ => unimplemented!()
                }
            }
            ExprST::Ternary {
                condition,
                consequence,
                alternative,
            } => {
                self.compile_expr(*condition);

                let jnt_operand_ptr = self.current_instructions().len() + 1;
                self.emit(&code::JumpNotTrue.make(u16::MAX));
                self.compile_expr(*consequence);
                let jump_operand_ptr = self.current_instructions().len() + 1;
                self.emit(&code::Jump.make(u16::MAX));
                let jnt_location = self.cur_ip();
                self.compile_expr(*alternative);

                self.overwrite_u16(jnt_operand_ptr, jnt_location);
                self.overwrite_u16(jump_operand_ptr, self.cur_ip());
            }
            ExprST::Switch { input, cases } => match input {
                Some(expr) => self.compile_match_switch(*expr, cases),
                None => self.compile_bool_switch(cases),
            },
            ExprST::Assign { left, right } => {
                self.compile_expr(*right);
                match left {
                    LHS::Ident { target, selectors: _ } => {
                        let i = self.symbol_map.register(target).index;
                        self.emit(&code::SetGVar.make(i))
                    },
                    _ => unimplemented!(),
                }
            }
            node => unimplemented!("Not sure how to compile {:?}", node),
        };
    }

    pub fn check(&self) -> BytecodeRef {
        BytecodeRef {
            instructions: self.current_instructions(),
            constants: &self.constants,
            global_count: self.symbol_map.size(),
        }
    }

    pub fn finish(self) -> Bytecode {
        let mut scopes = self.scopes;
        let instructions = scopes.pop().unwrap().instructions;
        Bytecode {
            instuctions: instructions.freeze(),
            constants: self.constants,
            global_count: self.symbol_map.size(),
        }
    }

    fn cur_ip(&self) -> u16 {
        self.current_instructions().len() as u16
    }

    fn emit(&mut self, bytes: &Bytes) {
        self.scopes
            .last_mut()
            .unwrap()
            .instructions
            .extend_from_slice(bytes);
    }

    fn emit_const(&mut self, const_ptr: usize) {
        self.emit(&code::Const.make(const_ptr as u16))
    }

    fn emit_binop(&mut self, binop: BinOp) {
        let bytes = match binop {
            BinOp::NullCoal => code::NullCoal.make(),
            BinOp::TupleStart => code::TupleStart.make(),
            BinOp::Exp => code::Exp.make(),
            BinOp::Mult => code::Mult.make(),
            BinOp::Inter => code::Inter.make(),
            BinOp::Div => code::Div.make(),
            BinOp::Mod => code::Mod.make(),
            BinOp::IntDiv => code::IntDiv.make(),
            BinOp::Add => code::Add.make(),
            BinOp::Subtract => code::Subtract.make(),
            BinOp::With => code::With.make(),
            BinOp::Less => code::Less.make(),
            BinOp::Union => code::Union.make(),
            BinOp::In => code::In.make(),
            BinOp::Notin => code::Notin.make(),
            BinOp::Subset => code::Subset.make(),
            BinOp::LT => code::Lt.make(),
            BinOp::LTEQ => code::Lteq.make(),
            BinOp::GT => code::Lt.make(),
            BinOp::GTEQ => code::Lteq.make(),
            BinOp::EQ => code::Eq.make(),
            BinOp::NEQ => code::Neq.make(),
            BinOp::And => code::And.make(),
            BinOp::Or => code::Or.make(),
            BinOp::Impl => code::Impl.make(),
            BinOp::Iff => code::Iff.make(),
        };
        self.emit(&bytes);
    }

    fn emit_preop(&mut self, preop: PreOp) {
        match preop {
            PreOp::Id => {} // No op, though this may change
            PreOp::Negate => self.emit(&code::Negate.make()),
            PreOp::DynVar => self.emit(&code::DynVar.make()),
            PreOp::Size => self.emit(&code::Size.make()),
            PreOp::Not => self.emit(&code::Not.make()),
        }
    }

    // OPTIMIZE: Constants could be added to a hashmap keyed by constant's value, and
    // the value of the hashmap would be an incrementing index which is returned by the
    // `add_const` fn. Before passing constant list to VM, constant map would be converted
    // to a vector. However, I think this may result in few space saves since programs
    // with many many similar constants aren't common.
    fn add_const(&mut self, constant: BaseObject) -> usize {
        self.constants.push(constant);
        self.constants.len() - 1
    }

    fn overwrite(&mut self, at: usize, value: Bytes) {
        let top_scope = self.scopes.last_mut().unwrap();
        for (i, byte) in value.into_iter().enumerate() {
            top_scope.instructions[at + i] = byte;
        }
    }

    fn overwrite_u16(&mut self, at: usize, value: u16) {
        let mut bytes = BytesMut::with_capacity(2);
        bytes.put_u16(value);
        self.overwrite(at, bytes.freeze())
    }

    fn compile_match_switch(&mut self, input: ExprST, cases: Vec<Case>) {
        self.compile_expr(input);
        self.emit(&code::PushMatch.make());
        self.compile_switch_cases(cases, code::JumpNotMatch.make(u16::MAX));
        self.emit(&code::PopMatch.make());
    }

    fn compile_bool_switch(&mut self, cases: Vec<Case>) {
        self.compile_switch_cases(cases, code::JumpNotTrue.make(u16::MAX));
    }

    fn compile_switch_cases(&mut self, cases: Vec<Case>, cond_jump_bytes: Bytes) {
        let mut jmp_operand_ptrs: Vec<usize> = vec![];
        let mut last_cond_jump_operand_ptr: Option<usize> = None;
        let mut has_default = false;
        
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
                last_cond_jump_operand_ptr = Some(self.ins_len() + 1);
                self.emit(&cond_jump_bytes);
                self.compile_expr_list(consequence, true);
                self.handle_null_return(null_return);
                jmp_operand_ptrs.push(self.ins_len() + 1);
                self.emit(&code::Jump.make(u16::MAX));
            } else {
                self.compile_expr_list(consequence, true);
                self.handle_null_return(null_return);
                // Any cases that follow the default case will not be compiled because they're unreachable
                has_default = true;
                break;
            }
        }

        if !has_default {
            // If no default case was provided, we jump to one that pushes null to the stack
            if let Some(ptr) = last_cond_jump_operand_ptr {
                self.overwrite_u16(ptr, self.cur_ip());
            }
            self.emit(&code::Null.make());
        }

        let cur_pos = self.cur_ip();
        for jmp_operand_ptr in jmp_operand_ptrs {
            self.overwrite_u16(jmp_operand_ptr, cur_pos);
        }
    }

    fn handle_null_return(&mut self, null_return: bool) {
        if null_return {
            self.emit(&code::Null.make());
        } else {
            let len = self.ins_len() - 1;
            self.scopes
                .last_mut()
                .unwrap()
                .instructions
                .truncate(len);
        }
    }

    fn compile_former(
        &mut self,
        former: Former,
        lit_builder: impl OpCodeMakeWithU16,
        range_builder: impl OpCodeMakeWithU16,
    ) {
        match former {
            Former::Literal(expressions) => {
                let size = expressions.len() as u16;
                self.compile_expr_list(expressions, false);
                self.emit(&lit_builder.make(size));
            }
            Former::Range { range_start, range_step, range_end } => {
                let parts = if range_step.is_none() { 2 } else { 3 };
                range_step.map(|step| self.compile_expr(*step));
                self.compile_expr(*range_end);
                self.compile_expr(*range_start);
                self.emit(&range_builder.make(parts));
            }
            _ => unimplemented!()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Bytecode, Compiler};
    use crate::code::code::{self, OpCode};
    use crate::code::debug::print_bytes;
    use crate::object::object::BaseObject::*;
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
        assert_eq!(int_code.instuctions, Bytes::from(vec![code::Const::VAL, 0, 0]));
        assert_eq!(int_code.constants[0], Integer(3));

        let float_code = compile("3.0");
        assert_eq!(float_code.instuctions, Bytes::from(vec![code::Const::VAL, 0, 0]));
        assert_eq!(float_code.constants[0], Float(3.0));

        let true_code = compile("true");
        assert_eq!(true_code.instuctions, Bytes::from(vec![code::True::VAL]));
        assert!(true_code.constants.is_empty());

        let false_code = compile("false");
        assert_eq!(false_code.instuctions, Bytes::from(vec![code::False::VAL]));
        assert!(false_code.constants.is_empty());

        let null_code = compile("null");
        assert_eq!(null_code.instuctions, Bytes::from(vec![code::Null::VAL]));
        assert!(null_code.constants.is_empty());

        let negative_int = compile("-1");
        assert_eq!(negative_int.instuctions, Bytes::from(vec![code::Const::VAL, 0, 0]));
        assert_eq!(negative_int.constants[0], Integer(-1));

        let negative_float = compile("-1.0");
        assert_eq!(negative_float.instuctions, Bytes::from(vec![code::Const::VAL, 0, 0]));
        assert_eq!(negative_float.constants[0], Float(-1.0));
    }

    #[test]
    fn simple_math() {
        assert_eq!(
            compile("3 + 4").instuctions,
            Bytes::from(vec![code::Const::VAL, 0, 0, code::Const::VAL, 0, 1, code::Add::VAL])
        );
        assert_eq!(
            compile("3 - 4").instuctions,
            Bytes::from(vec![code::Const::VAL, 0, 0, code::Const::VAL, 0, 1, code::Subtract::VAL])
        );
        assert_eq!(
            compile("3 + (4 / 5)").instuctions,
            Bytes::from(vec![code::Const::VAL, 0, 0, code::Const::VAL, 0, 1, code::Const::VAL, 0, 2, code::Div::VAL, code::Add::VAL])
        );
    }

    #[test] #[rustfmt::skip]
    fn ternary() {
        assert_bytes(&compile_program("if true ? 1 : 2; 99;"), vec![
            // 0
            code::True::VAL,
            // 1
            code::JumpNotTrue::VAL, 0, 10,
            // 4
            code::Const::VAL, 0, 0,
            // 7
            code::Jump::VAL, 0, 13,
            // 10
            code::Const::VAL, 0, 1,
            // 13
            code::Pop::VAL,
            // 14
            code::Const::VAL, 0, 2,
            // 17
            code::Pop::VAL,
            // 18
        ]);
    }
}
