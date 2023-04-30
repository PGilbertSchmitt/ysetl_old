use pest::Parser;
use pest_derive::Parser;
use pest::error::Error;
use pest::iterators::Pair;

use super::ast::ExprST;

#[derive(Parser)]
#[grammar="parser/ysetl.pest"]
struct YsetlParser;

pub fn parse_program(input: &str) -> Result<(), Error<Rule>> {
    let program = YsetlParser::parse(Rule::program_input, input)?.next().unwrap();

    match program.as_rule() {
        Rule::program => {
            // The program rule captures the program name first, followed by all expressions (separated by semicolons)
            let mut inner = program.into_inner();
            let name_node = inner
                .next()
                .unwrap();
            let program_name = atom_value(name_node);

            println!(
                "Program '{:?}' contains expressions of types: {:?}",
                program_name,
                inner.map(|p| parse_expr(p)).collect::<Vec<_>>()
            );

        },
        Rule::program_missing_expr => {
            println!("Program must have at least one expression");
        },
        _ => unreachable!(),
    } 

    Ok(())
}

fn atom_value(atom_pair: Pair<Rule>) -> &str {
    atom_pair.into_inner().next().unwrap().as_str()
}

fn string_value(string_pair: Pair<Rule>) -> &str {
    string_pair.into_inner().next().unwrap().as_str()
}

fn number_value(number_pair: Pair<Rule>) -> ExprST {
    let mut number_parts = number_pair.into_inner().map(|p| p.as_str());
    construct_number(
        number_parts.next().unwrap(),
        number_parts.next().unwrap(),
        number_parts.next().unwrap(),
        number_parts.next().unwrap(),
    )
}

fn construct_number(
    dash: &str,
    base: &str,
    decimal: &str,
    exp: &str,
) -> ExprST<'static> {
    let mut is_float = false;
    let mut number_str = dash.to_owned();
    number_str = number_str + base;

    if decimal != "" {
        is_float = true;
        number_str.push_str(decimal)
    }

    if exp != "" {
        is_float = true;
        number_str.push('e');
        number_str.push_str(&exp[1..]);
    }

    if is_float {
        ExprST::Float(number_str.parse().unwrap())
    } else {
        ExprST::Integer(number_str.parse().unwrap())
    }
}

#[allow(dead_code)]
fn inspect(pair: Pair<Rule>) -> ExprST {
    println!("{:?}", pair);
    ExprST::Null
}

fn parse_expr(input: Pair<Rule>) -> Result<ExprST, String> {
    match input.as_rule() {
        Rule::null => Ok(ExprST::Null),
        Rule::newat => Ok(ExprST::Newat),
        Rule::true_ => Ok(ExprST::True),
        Rule::false_ => Ok(ExprST::False),
        Rule::atom => Ok(ExprST::Atom(atom_value(input))),
        Rule::string => Ok(ExprST::String(string_value(input))),
        Rule::ident => Ok(ExprST::Ident(input.as_str())),
        Rule::number => Ok(number_value(input)),
        _ => {
           Err(format!("Unexpected expression type: {:?}", input.as_rule()))
        }
    }
}
