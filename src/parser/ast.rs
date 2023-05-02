#[derive(Debug)]
pub enum BinOp {
    NullCoal,
    TupleStart,
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
pub enum PreOp {
    Negate,
    Id,
    DynVar, // Dynamic variable
    Size,
    Not,
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
    Infix{op: BinOp, left: Box<ExprST<'a>>, right: Box<ExprST<'a>>},
    ReduceWithOp{op: BinOp, left: Box<ExprST<'a>>, right: Box<ExprST<'a>>},
    ReduceWithExpr{apply: Box<ExprST<'a>>, left: Box<ExprST<'a>>, right: Box<ExprST<'a>>},
    InfixInject{apply: Box<ExprST<'a>>, left: Box<ExprST<'a>>, right: Box<ExprST<'a>>},
    Prefix{op: PreOp, right: Box<ExprST<'a>>},
    Call{left: Box<ExprST<'a>>}, // Needs info about selectors
}