use pest::Parser;
use pest::error::Error;
use pest::iterators::{Pair,Pairs};
use pest::pratt_parser::PrattParser;
use lazy_static;

use super::grammar::YsetlParser;
use super::grammar::Rule;
use super::ast::{ExprST, PreOp, Former, IteratorST, IteratorType, Bound};
use super::ast::BinOp;
use super::debug::{pair_str};

type ExprResult<'a> = Result<ExprST<'a>, String>;

lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Op, Assoc};
        
        PrattParser::new()
            .op(Op::prefix(Rule::not)) // Same as Rule::bang, but with much lower precedence
            .op(Op::infix(Rule::infix_inject, Assoc::Left))
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
            .op(Op::prefix(Rule::dash_pre) |
                Op::prefix(Rule::plus_pre) |
                Op::prefix(Rule::at_pre) |
                Op::prefix(Rule::hash) |
                Op::prefix(Rule::bang)
            )
            .op(Op::postfix(Rule::fn_call) |
                Op::postfix(Rule::range_call) |
                Op::postfix(Rule::pick_call)
            )
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

            for pair in inner { 
                // println!("{:?}", pair);
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
    )
}

/* 
 * This seems a little silly, but YSETL's float literals are ALMOST the same as Rust's,
 * with the only exception being that the exponent marker can be 'e', 'E', 'f', or 'F'.
 * There is no semantic difference between these markers, it's up to personal preference.
 */
fn construct_number(
    base: &str,
    decimal: &str,
    exp: &str,
) -> ExprST<'static> {
    let mut is_float = false;
    let mut number_str = base.to_owned();

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

fn pull_param_type<'a>(rule: Rule, params: &mut Pairs<'a, Rule>) -> Vec<&'a str> {
    let mut output_vec = Vec::new();
    while let Some(param_str) = params.peek().and_then(|param_type| {
            if param_type.as_rule() == rule {
                Some(params.next().unwrap().into_inner().next().unwrap().as_str())
            } else {
                None
            }
    }) {
        output_vec.push(param_str)
    }
    output_vec
}

struct ParamLists<'a>(Vec<&'a str>,Vec<&'a str>,Vec<&'a str>);

fn parse_param_list<'a>(param_list: Pair<'a, Rule>) -> Result<ParamLists, String> {
    let mut params = param_list.into_inner();
    
    let req_params = pull_param_type(Rule::req_param, &mut params);
    let opt_params = pull_param_type(Rule::opt_param, &mut params);
    let locked_params = pull_param_type(Rule::locked_param, &mut params);

    if let Some(param) = params.next() {
        Err(format!("Unexpected param {}, params must be ordered correctly", param.as_str()))
    } else {
        Ok(ParamLists(req_params, opt_params, locked_params))
    }
}

fn parse_func<'a>(func_pair: Pair<'a, Rule>) -> ExprResult<'a> {
    let mut parts = func_pair.into_inner();
    let ParamLists(
        req_params,
        opt_params,
        locked_params
    ) = parse_param_list(parts.next().unwrap())?;

    let mut body = Vec::new();
    while let Some(expr) = parts.peek().and_then(|part_type| {
        if part_type.as_rule() != Rule::captured_semicolon {
            parts.next()
        } else {
            None
        }
    }) {
        body.push(parse_expr(expr)?)
    }

    let null_return = parts.next().is_some();

    Ok(ExprST::Function {
        req_params,
        opt_params,
        locked_params,
        body,
        null_return,
    })
}

#[allow(dead_code)]
fn inspect(input: Pair<Rule>) -> ExprResult {
    println!("{}", pair_str(input));
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

fn to_prefix<'a>(rhs: ExprResult<'a>, op: PreOp) -> ExprResult<'a> {
    Ok(ExprST::Prefix { op, right: Box::new(rhs?) })
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

fn to_infix_inject<'a>(lhs: ExprResult<'a>, rhs: ExprResult<'a>, op: Pair<'a, Rule>) -> ExprResult<'a> {
    let inner_op = op.into_inner().next().unwrap();
    Ok(ExprST::InfixInject {
        apply: Box::new(map_primary_to_expr(inner_op).unwrap()),
        left: Box::new(lhs?),
        right: Box::new(rhs?)
    })
}

fn parse_bound<'a>(bound: Pair<'a, Rule>) -> Bound<'a> {
    match bound.as_rule() {
        Rule::tilde => Bound::Tilde,
        Rule::ident => Bound::Ident(bound.as_str()),
        Rule::bound_list => Bound::List(bound.into_inner().map(parse_bound).collect()),
        _ => unreachable!(),
    }
}

fn parse_iterator_list_item<'a>(item: Pair<'a, Rule>) -> IteratorType<'a> {
    let rule = item.as_rule();
    let mut inner = item.into_inner();
    if rule == Rule::in_iterator {
        let list = inner.next().unwrap().into_inner().map(parse_bound).collect();
        inner.next(); // Shed the unwanted "in" that's parsed (too lazy to make it silent...)
        let expr = Box::new(parse_expr(inner.next().unwrap()).unwrap());
        return IteratorType::In { list, expr }
    }
    let bound = parse_bound(inner.next().unwrap());
    inner.next(); // Shed the unwanted "=" (wow, that's lazy)
    let collection_ident = inner.next().unwrap().as_str();
    let list = inner.next().unwrap().into_inner().map(parse_bound).collect();
    match rule {
        Rule::select_iterator_single => IteratorType::SelectSingle {
            bound,
            collection_ident,
            list,
        },
        Rule::select_iterator_multi => IteratorType::SelectMulti {
            bound,
            collection_ident,
            list,
        },
        _ => unreachable!(),
    }
}

fn parse_iterator<'a>(mut iterator_parts: Pairs<'a, Rule>) -> IteratorST<'a> {
    IteratorST {
        iterators: iterator_parts.next().unwrap().into_inner().map(parse_iterator_list_item).collect(),
        filter: iterator_parts.map(|expr| parse_expr(expr).unwrap()).collect(),
    }
}

fn parse_former<'a>(mut former: Pairs<'a, Rule>) -> Former<'a> {
    former.next().map_or_else(|| Former::Literal(vec![]), |former_type| {
        let rule = former_type.as_rule();
        let mut former_parts = former_type.into_inner();
        match rule {
            Rule::literal_former => {
                Former::Literal(
                    former_parts
                        .map(|expr| parse_expr(expr).unwrap())
                        .collect()
                )
            },
            Rule::range_former => {
                let range_start = unwrap_range(former_parts.next().unwrap())
                    .expect(&format!("Range in collection former must be well defined"));
                let range_end = unwrap_range(former_parts.next().unwrap())
                    .expect(&format!("Range in collection former must be well defined"));
                Former::Range { range_start, range_step: None, range_end }
            },
            Rule::interval_range_former => {
                let range_start = Box::new(parse_expr(former_parts.next().unwrap()).unwrap());
                let range_step = Some(unwrap_range(former_parts.next().unwrap())
                    .expect(&format!("Range in collection former must be well defined")));
                let range_end = unwrap_range(former_parts.next().unwrap())
                    .expect(&format!("Range in collection former must be well defined"));
                Former::Range { range_start, range_step, range_end }
            },
            Rule::iterator_former => {
                let output = Box::new(parse_expr(former_parts.next().unwrap()).unwrap());
                let iterator = parse_iterator(former_parts.next().unwrap().into_inner());
                Former::Iterator {
                    iterator,
                    output,
                }
            },
            _ => unreachable!(),
        }
    })
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
        Rule::tuple_literal => Ok(ExprST::TupleLiteral(parse_former(primary.into_inner()))),
        Rule::set_literal => Ok(ExprST::SetLiteral(parse_former(primary.into_inner()))),
        Rule::short_func => parse_func(primary),
        Rule::long_func => parse_func(primary),
        Rule::nested_expression => parse_expr(primary.into_inner().next().unwrap()),
        rule => unreachable!("parse_expr expected primary, received {:?}", rule),
    }
}

fn unwrap_expr_list<'a>(list: Pair<'a, Rule>) -> Vec<ExprST> {
    list.into_inner().map(|part| parse_expr(part).unwrap()).collect::<Vec<_>>()
}

fn to_call_expr<'a>(lhs: ExprResult<'a>, postfix: Pair<'a, Rule>) -> ExprResult<'a> {
    Ok(ExprST::Call {
        left: Box::new(lhs?),
        args: unwrap_expr_list(postfix),
    })
}

fn unwrap_range(range_part: Pair<Rule>) -> Option<Box<ExprST>> {
    range_part.into_inner().next().map(|part| {
        Box::new(parse_expr(part).unwrap())
    })
}

fn to_range_expr<'a>(lhs: ExprResult<'a>, postfix: Pair<'a, Rule>) -> ExprResult<'a> {
    let mut ranges = postfix.into_inner();
    let range_start = unwrap_range(ranges.next().unwrap());
    let range_end = unwrap_range(ranges.next().unwrap());
    return Ok(ExprST::Range {
        left: Box::new(lhs?),
        range_start,
        range_end,
    })
}

fn to_pick_expr<'a>(lhs: ExprResult<'a>, postfix: Pair<'a, Rule>) -> ExprResult<'a> {
    Ok(ExprST::Pick {
        left: Box::new(lhs?),
        picks: unwrap_expr_list(postfix),
    })
}

fn parse_bin_expr(input: Pair<Rule>) -> ExprResult {
    PRATT_PARSER
        .map_primary(map_primary_to_expr)
        .map_prefix(|prefix, rhs| {
            match prefix.as_rule() {
                Rule::dash_pre => to_prefix(rhs, PreOp::Negate),
                Rule::plus_pre => to_prefix(rhs, PreOp::Id),
                Rule::at_pre => to_prefix(rhs, PreOp::DynVar),
                Rule::hash => to_prefix(rhs, PreOp::Size),
                Rule::bang => to_prefix(rhs, PreOp::Not),
                Rule::not => to_prefix(rhs, PreOp::Not),
                rule => unreachable!("parse_expr expected prefix expression, received {:?}", rule),
            }
        })
        .map_postfix(|lhs, postfix| {
            match postfix.as_rule() {
                Rule::fn_call => to_call_expr(lhs, postfix),
                Rule::range_call => to_range_expr(lhs, postfix),
                Rule::pick_call => to_pick_expr(lhs, postfix),
                rule => unreachable!("parse_expr expected postfix expression, received {:?}", rule),
            }
        })
        .map_infix(|lhs, op, rhs| {
            let op_rule = op.as_rule();
            match op_rule {
                // Normal Rules
                | Rule::plus
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
                Rule::reduce_op => to_reduce_expr(lhs, rhs, op),
                Rule::infix_inject => to_infix_inject(lhs, rhs, op),
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
