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
}