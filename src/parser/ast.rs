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
    In,
    Notin,
    Subset,
    LT,
    LTEQ,
    GT,
    GTEQ,
    EQ,
    NEQ,
    And,
    Or,
    Impl,
    Iff,
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
pub enum Bound<'a> {
    Tilde,
    Ident(&'a str),
    List(BoundList<'a>),
}

type BoundList<'a> = Vec<Bound<'a>>;

#[derive(Debug)]
pub enum IteratorType<'a> {
    In {
        list: BoundList<'a>,
        expr: Box<ExprST<'a>>,
    },
    SelectSingle {
        bound: Bound<'a>,
        collection_ident: &'a str,
        list: BoundList<'a>,
    },
    SelectMulti {
        bound: Bound<'a>,
        collection_ident: &'a str,
        list: BoundList<'a>,
    },
}

#[derive(Debug)]
pub struct IteratorST<'a> {
    pub iterators: Vec<IteratorType<'a>>,
    pub filter: Vec<ExprST<'a>>,
}

#[derive(Debug)]
pub enum Former<'a> {
    Literal(Vec<ExprST<'a>>),
    Range {
        range_start: Box<ExprST<'a>>,
        range_step: Option<Box<ExprST<'a>>>,
        range_end: Box<ExprST<'a>>,
    },
    Iterator {
        iterator: IteratorST<'a>,
        output: Box<ExprST<'a>>,
    },
}

#[derive(Debug)]
pub enum Selector<'a> {
    Call(Vec<ExprST<'a>>),
    Range(Option<Box<ExprST<'a>>>, Option<Box<ExprST<'a>>>),
    Pick(Vec<ExprST<'a>>),
}

#[derive(Debug)]
pub enum LHS<'a> {
    Tilde,
    Ident {
        target: &'a str,
        selectors: Vec<Selector<'a>>,
    },
    List(Vec<LHS<'a>>),
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
    TupleLiteral(Former<'a>),
    SetLiteral(Former<'a>),
    Function {
        req_params: Vec<&'a str>,
        opt_params: Vec<&'a str>,
        locked_params: Vec<&'a str>,
        body: Vec<ExprST<'a>>,
        null_return: bool,
    },
    Infix {
        op: BinOp,
        left: Box<ExprST<'a>>,
        right: Box<ExprST<'a>>,
    },
    ReduceWithOp {
        op: BinOp,
        left: Box<ExprST<'a>>,
        right: Box<ExprST<'a>>,
    },
    ReduceWithExpr {
        apply: Box<ExprST<'a>>,
        left: Box<ExprST<'a>>,
        right: Box<ExprST<'a>>,
    },
    InfixInject {
        apply: Box<ExprST<'a>>,
        left: Box<ExprST<'a>>,
        right: Box<ExprST<'a>>,
    },
    Prefix {
        op: PreOp,
        right: Box<ExprST<'a>>,
    },
    Postfix {
        left: Box<ExprST<'a>>,
        selector: Selector<'a>,
    },
    Assign {
        left: LHS<'a>,
        right: Box<ExprST<'a>>,
    },
}
