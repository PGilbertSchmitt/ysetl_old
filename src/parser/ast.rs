#[derive(Debug)]
pub enum Op {
    NullCoal,
    At,
    Exp,
    Mult,
    Inter,
    Div,
    Mod,
    IntDiv,
    Add,
    Subtract,
    With,
    Less,
    Union,
}

#[derive(Debug)]
pub enum ExprST<'a> {
    Null,
    Newat,
    True,
    False,
    Atom(&'a str),
    String(&'a str),
    Ident(&'a str),
    Integer(i64),
    Float(f64),
    Infix{op: Op, left: Box<ExprST<'a>>, right: Box<ExprST<'a>>},
    ReduceWithOp{op: Op, left: Box<ExprST<'a>>, right: Box<ExprST<'a>>},
    ReduceWithExpr{apply: Box<ExprST<'a>>, left: Box<ExprST<'a>>, right: Box<ExprST<'a>>},
}