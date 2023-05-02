use pest::Parser;
use pest_derive::Parser;
use pest::error::Error;
use pest::iterators::{Pair};
use pest::pratt_parser::PrattParser;
use lazy_static;

use super::ast::ExprST;
use super::ast::BinOp;

#[derive(Parser)]
#[grammar="parser/ysetl.pest"]
struct YsetlParser;

type ExprResult<'a> = Result<ExprST<'a>, String>;

lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Op, Assoc};
        
        PrattParser::new()
            .op(Op::infix(Rule::plus, Assoc::Left) |
                Op::infix(Rule::dash, Assoc::Left) |
                Op::infix(Rule::with, Assoc::Left) |
                Op::infix(Rule::less, Assoc::Left) |
                Op::infix(Rule::union_, Assoc::Left)
            )
            .op(Op::infix(Rule::star, Assoc::Left) |
                Op::infix(Rule::slash, Assoc::Left) |
                Op::infix(Rule::mod_, Assoc::Left) |
                Op::infix(Rule::div, Assoc::Left) |
                Op::infix(Rule::inter, Assoc::Left)
            )
            .op(Op::infix(Rule::dbl_star, Assoc::Right))
            .op(Op::infix(Rule::reduce_op, Assoc::Right))
            .op(Op::infix(Rule::dbl_qst, Assoc::Right))
            .op(Op::infix(Rule::at, Assoc::Right))
    };
}

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
            println!("Rest of program\n\n{:?}\n\n", inner);
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
fn inspect(input: Pair<Rule>) -> ExprResult {
    println!("{:?}", input);
    Ok(ExprST::Null)
}

fn to_binop(rule: Rule) -> BinOp {
    match rule {
        Rule::plus => BinOp::Add,
        Rule::dash => BinOp::Subtract,
        Rule::with => BinOp::With,
        Rule::less => BinOp::Less,
        Rule::union_ => BinOp::Union,
        Rule::star => BinOp::Mult,
        Rule::slash => BinOp::Div,
        Rule::mod_ => BinOp::Mod,
        Rule::div => BinOp::IntDiv,
        Rule::inter => BinOp::Inter,
        Rule::dbl_star => BinOp::Exp,
        Rule::dbl_qst => BinOp::NullCoal,
        Rule::at => BinOp::TupleStart, 
        _ => unreachable!("Expected pure binary operator, received {:?}", rule),
    }
}

fn to_infix<'a>(lhs: ExprResult<'a>, rhs: ExprResult<'a>, op: BinOp) -> ExprResult<'a> {
    Ok(ExprST::Infix {
        op: op,
        left: Box::new(lhs?),
        right: Box::new(rhs?),
    })
}

fn to_reduce_expr<'a>(lhs: ExprResult<'a>, rhs: ExprResult<'a>, op: Pair<'a, Rule>) -> ExprResult<'a> {
    let inner_op = op.into_inner().next().unwrap();
    let left = Box::new(lhs?);
    let right = Box::new(rhs?);
    match inner_op.as_rule() {
        Rule::nested_expression | Rule::ident => Ok(ExprST::ReduceWithExpr {
            apply: Box::new(map_primary_to_expr(inner_op).unwrap()),
            left,
            right,
        }),
        bin_op => Ok(ExprST::ReduceWithOp {
            op: to_binop(bin_op),
            left,
            right,
        })
    }
}

fn map_primary_to_expr(primary: Pair<Rule>) -> ExprResult {
    match primary.as_rule() {
        Rule::null => Ok(ExprST::Null),
        Rule::newat => Ok(ExprST::Newat),
        Rule::true_ => Ok(ExprST::True),
        Rule::false_ => Ok(ExprST::False),
        Rule::atom => Ok(ExprST::Atom(atom_value(primary))),
        Rule::string => Ok(ExprST::String(string_value(primary))),
        Rule::ident => Ok(ExprST::Ident(primary.as_str())),
        Rule::number => Ok(number_value(primary)),
        Rule::nested_expression => parse_expr(primary.into_inner().next().unwrap()),
        rule => unreachable!("parse_expr expected primary, received {:?}", rule),
    }
}

fn parse_bin_expr(input: Pair<Rule>) -> ExprResult {
    PRATT_PARSER
        .map_primary(map_primary_to_expr)
        .map_infix(|lhs, op, rhs| {
            let op_rule = op.as_rule();
            match op_rule {
                // Normal Rules
                  Rule::plus
                | Rule::dash
                | Rule::with
                | Rule::less
                | Rule::union_
                | Rule::star
                | Rule::slash
                | Rule::mod_
                | Rule::div
                | Rule::inter
                | Rule::dbl_star
                | Rule::dbl_qst
                | Rule::at => to_infix(lhs, rhs, to_binop(op_rule)),

                // Special operator infix
                // Rule::reduce_op => inspect(op),
                Rule::reduce_op => to_reduce_expr(lhs, rhs, op),
                rule => unreachable!("parse_expr expected infix expression, received {:?}", rule),
            }
        })
        .parse(input.into_inner())
}

fn parse_expr(input: Pair<Rule>) -> ExprResult {
    match input.as_rule() {
        // There will be non-binop expressions that go here
        Rule::bin_expr => parse_bin_expr(input),
        _ => unreachable!(),
    }
}
