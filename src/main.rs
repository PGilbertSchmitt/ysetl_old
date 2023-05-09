use std::fs;
use compiler::compiler::Compiler;
use parser::parser::parse_from_expr;
use vm::vm::VM;

pub mod code;
pub mod compiler;   
pub mod object;
pub mod parser;
pub mod vm;

static INPUT_PATH: &'static str = "expr.ysetl";

fn main() {
    let input = fs::read_to_string(INPUT_PATH).unwrap();
    let expr = parse_from_expr(&input).unwrap();
    let mut compiler = Compiler::new();
    compiler.compile(expr);
    let bc = compiler.finish();
    let mut vm = VM::new(bc);
    let result = vm.run();  
    println!("\nExpression resulted in {:?}", result);
}
