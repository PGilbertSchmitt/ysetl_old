#[derive(Debug)]
pub enum Op {
    At,
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
    Infix(Op, Box<ExprST<'a>>, Box<ExprST<'a>>),
}