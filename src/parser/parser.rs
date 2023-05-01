use pest::Parser;
use pest_derive::Parser;
use pest::error::Error;
use pest::iterators::{Pair, Pairs};

use super::ast::ExprST;
use super::ast::Op;

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

            println!("Executing program '{}'", program_name);
            for pair in inner {
                println!("{} -> {:?}", pair.as_str(), parse_expr(pair));
            }
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

/* 
 * This seems a little silly, but YSETL's float literals are ALMOST the same as rusts,
 * with the only exception being that the exponent marker can be 'e', 'E', 'f', or 'F'.
 */
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
fn inspect(input: Pair<Rule>) -> Result<ExprST, String> {
    println!("{:?}", input);
    Ok(ExprST::Null)
}

fn to_op(input: &Pair<Rule>) -> Option<Op> {
    let rule = input.as_rule();
    match rule {
        Rule::at => Some(Op::At),
        Rule::dbl_qst => Some(Op::NullCoal),
        Rule::dbl_star => Some(Op::Exp),
        Rule::star => Some(Op::Mult),
        Rule::inter => Some(Op::Inter),
        Rule::slash => Some(Op::Div),
        Rule::div => Some(Op::IntDiv),
        Rule::mod_ => Some(Op::IntDiv),
        Rule::plus => Some(Op::Add),
        Rule::dash => Some(Op::Subtract),
        Rule::with => Some(Op::With),
        Rule::less => Some(Op::Less),
        Rule::union_ => Some(Op::Union),
        _ => None,
    }
}

fn parse_nonassoc_infix(mut parts: Pairs<Rule>) -> ExprST {
    let first = parse_expr(parts.next().unwrap()).unwrap();
    match parts.next() {
        Some(op_rule) => ExprST::Infix {
            op: to_op(&op_rule).unwrap(),
            left: Box::new(first),
            right: Box::new(parse_expr(parts.next().unwrap()).unwrap()),
            // Maybe check if there are more parts?
        },
        None => first,
    }
}

fn parse_right_assoc_infix(mut parts: Pairs<Rule>) -> ExprST {
    let first = parse_expr(parts.next().unwrap()).unwrap();
    match parts.next() {
        Some(op_rule) => ExprST::Infix {
            op: to_op(&op_rule).unwrap(),
            left: Box::new(first),
            right: Box::new(parse_right_assoc_infix(parts)),
        },
        None => first,
    }
}

fn parse_left_assoc_infix(mut parts: Pairs<Rule>) -> ExprST {
    let last = parse_expr(parts.next_back().unwrap()).unwrap();
    match parts.next_back() {
        Some(op_rule) => ExprST::Infix {
            op: to_op(&op_rule).unwrap(),
            left: Box::new(parse_left_assoc_infix(parts)),
            right: Box::new(last),
        },
        None => last
    }
}

// More specific than the general non-assoc parse handler
fn parse_reduce_infix(mut parts: Pairs<Rule>) -> ExprST {
    let first = parse_expr(parts.next().unwrap()).unwrap();
    match parts.next() {
        Some(_) => {
            let operation = parts.next().unwrap();
            match to_op(&operation) {
                Some(op) => ExprST::ReduceWithOp {
                    op: op,
                    left: Box::new(first),
                    right: Box::new(parse_expr(parts.next().unwrap()).unwrap()),
                },  
                // If `to_op` doesn't catch the operation pair, than it's an infix binop, and
                // the operation is actually just a dot which can be ignored in favor of the
                // following token
                None => ExprST::ReduceWithExpr {
                    apply: Box::new(parse_expr(parts.next().unwrap()).unwrap()),
                    left: Box::new(first),
                    right: Box::new(parse_expr(parts.next().unwrap()).unwrap()),
                },
            }
        },
        None => first,
    }
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
        Rule::null_coal_expr => Ok(parse_right_assoc_infix(input.into_inner())),
        Rule::tuple_start_expr => Ok(parse_nonassoc_infix(input.into_inner())),
        Rule::reduce_expr => Ok(parse_reduce_infix(input.into_inner())),
        Rule::exponent_expr => Ok(parse_right_assoc_infix(input.into_inner())),
        Rule::mult_expr => Ok(parse_left_assoc_infix(input.into_inner())),
        Rule::add_expr => Ok(parse_left_assoc_infix(input.into_inner())),
        _ => {
           Err(format!("Unexpected expression type: {:?}", input.as_rule()))
        }
    }
}
