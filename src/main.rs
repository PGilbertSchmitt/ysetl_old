use std::fs;
use parser::parser::parse_program;

pub mod parser;

fn main() {
    let input = fs::read_to_string("rubric.ysetl").expect("Error opening file");
    parse_program(&input).unwrap();
}
