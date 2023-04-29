use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar="parser/ysetl.pest"]
struct YsetlParser;

pub fn parse(input: &str) {
    let pre_parse = YsetlParser::parse(Rule::Input, input).unwrap();
    println!("{:?}", pre_parse);
}
