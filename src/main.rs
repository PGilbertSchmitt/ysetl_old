use std::fs;
use parser::parser::parse;

pub mod parser;

fn main() {
    let input = fs::read_to_string("rubric.ysetl").expect("Error opening file");
    parse(&input);
}
