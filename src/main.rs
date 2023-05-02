use std::fs;
use parser::parser::parse_program;

pub mod parser;

static INPUT_PATH: &'static str = "rubric.ysetl";

fn main() {
    let input = fs::read_to_string(INPUT_PATH).expect("Error opening file");
    parse_program(&input).unwrap();
}
