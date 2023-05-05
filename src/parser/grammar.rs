use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "parser/ysetl.pest"]
pub struct YsetlParser;

#[cfg(test)]
mod tests {
    use super::Rule;
    use super::YsetlParser;
    use pest::Parser;

    fn parse_is_ok(rule: Rule, input: &str) {
        match YsetlParser::parse(rule, input) {
            Ok(_) => assert!(true),
            Err(err) => assert!(false, "{:?}", err),
        }
    }

    #[test]
    fn primaries() {
        parse_is_ok(Rule::null, "null");
        parse_is_ok(Rule::newat, "newat");
        parse_is_ok(Rule::false_, "false");
        parse_is_ok(Rule::true_, "true");

        parse_is_ok(Rule::atom, ":abc");

        parse_is_ok(Rule::number, "1");
        parse_is_ok(Rule::number, "123.456");
        parse_is_ok(Rule::number, "1.23456e-2");
        parse_is_ok(Rule::number, "01e2");
        parse_is_ok(Rule::number, "01f2");
        parse_is_ok(Rule::number, "01E2");
        parse_is_ok(Rule::number, "01F2");

        parse_is_ok(Rule::string, "\"hello, world\"");
        parse_is_ok(Rule::string, "\"Hello. \\nWorld.\"");
        parse_is_ok(Rule::string, "\"hello, \\\"world\\\"\"");

        parse_is_ok(Rule::ident, "foo");
    }

    #[test]
    fn prefix_expression() {
        parse_is_ok(Rule::expr, "-foo");
        parse_is_ok(Rule::expr, "+foo");
        parse_is_ok(Rule::expr, "@foo");
        parse_is_ok(Rule::expr, "#foo");
        parse_is_ok(Rule::expr, "!foo");
        parse_is_ok(Rule::expr, "not foo");
    }

    #[test]
    fn postfix_expression() {
        parse_is_ok(Rule::expr, "foo()");
        parse_is_ok(Rule::expr, "foo(a)");
        parse_is_ok(Rule::expr, "foo(a,b+c)");

        parse_is_ok(Rule::expr, "foo(..)");
        parse_is_ok(Rule::expr, "foo(a..)");
        parse_is_ok(Rule::expr, "foo(..b)");
        parse_is_ok(Rule::expr, "foo(a..b)");

        parse_is_ok(Rule::expr, "foo(a*2 .. b()/2)");
    }

    #[test]
    fn bin_expression() {
        parse_is_ok(Rule::expr, "a @ b @ c");
        parse_is_ok(Rule::expr, "a ?? b ?? c");
        parse_is_ok(Rule::expr, "a %+ b");
        parse_is_ok(Rule::expr, "a %foo b");
        parse_is_ok(Rule::expr, "a %(foo) b");
        parse_is_ok(Rule::expr, "a ** b ** c");
        parse_is_ok(Rule::expr, "a * b * c");
        parse_is_ok(Rule::expr, "a / b / c");
        parse_is_ok(Rule::expr, "a mod b mod c");
        parse_is_ok(Rule::expr, "a div b div c");
        parse_is_ok(Rule::expr, "a inter b inter c");
        parse_is_ok(Rule::expr, "a + b + c");
        parse_is_ok(Rule::expr, "a - b - c");
        parse_is_ok(Rule::expr, "a with b with c");
        parse_is_ok(Rule::expr, "a less b less c");
        parse_is_ok(Rule::expr, "a union b union c");
        parse_is_ok(Rule::expr, "a .foo b");
        parse_is_ok(Rule::expr, "a .(foo) b");
    }

    #[test]
    fn iterator() {
        parse_is_ok(Rule::iterator, "x in Z");
        parse_is_ok(Rule::iterator, "x=f(y)");
        parse_is_ok(Rule::iterator, "[x,y]=f(z)");
        parse_is_ok(Rule::iterator, "x=Z(y),a=C(b)");
        parse_is_ok(Rule::iterator, "x in Z | not x");
    }

    #[test]
    fn set_literal() {
        parse_is_ok(Rule::set_literal, "{}");
        parse_is_ok(Rule::set_literal, "{1}");
        parse_is_ok(Rule::set_literal, "{1,2}");
        parse_is_ok(Rule::set_literal, "{1..10}");
        parse_is_ok(Rule::set_literal, "{1,3..10}");
        parse_is_ok(Rule::set_literal, "{x+2 : x in Z}");
        parse_is_ok(Rule::set_literal, "{[x,y] : x in Z, y=W(x)}");
    }

    #[test]
    fn tuple_literal() {
        parse_is_ok(Rule::tuple_literal, "[]");
        parse_is_ok(Rule::tuple_literal, "[1]");
        parse_is_ok(Rule::tuple_literal, "[1,2]");
        parse_is_ok(Rule::tuple_literal, "[1..10]");
        parse_is_ok(Rule::tuple_literal, "[1,3..10]");
        parse_is_ok(Rule::tuple_literal, "[x+2 : x in Z]");
        parse_is_ok(Rule::tuple_literal, "[[x,y] : x in Z, y=W(x) | not x]");
    }

    #[test]
    fn functions() {
        parse_is_ok(Rule::short_func, "() => 5");
        parse_is_ok(Rule::short_func, "(a) => a + 5");
        parse_is_ok(Rule::short_func, "(a,b?) => a + (b ?? 5)");
        parse_is_ok(Rule::short_func, "(a,b?,c!) => a + (b ?? c)");

        parse_is_ok(Rule::long_func, "func () { 5 }");
        parse_is_ok(Rule::long_func, "func (a) { a + 5; a - 5 }");
        parse_is_ok(Rule::long_func, "func (a) { a + 5; a - 5; }");
        parse_is_ok(Rule::long_func, "func (a,b?,c!) { foo(a); a + (b ?? c) }");
    }
}
