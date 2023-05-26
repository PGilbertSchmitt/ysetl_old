use std::fs;
use compiler::compiler::Compiler;
use parser::parser::parse_from_program;
use vm::vm::VM;

// use crate::code::debug::print_bytes;

pub mod code;
pub mod compiler;   
pub mod object;
pub mod parser;
pub mod vm;

static INPUT_PATH: &'static str = "program.ysetl";

fn main() {
    let input = fs::read_to_string(INPUT_PATH).unwrap();
    let expr = parse_from_program(&input).unwrap();
    let mut compiler = Compiler::new();
    compiler.compile_program(expr);
    let bc = compiler.finish();
    // println!("{}", print_bytes(&bc.instuctions));
    let mut vm = VM::new(bc);
    println!("Last pop: {:?}", vm.run());
}
