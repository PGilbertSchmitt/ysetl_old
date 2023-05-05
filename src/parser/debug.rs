use super::grammar::Rule;
use pest::iterators::{Pair, Pairs};

pub fn pair_str(rule: Pair<Rule>) -> String {
    format!(
        r#"{{ "rule": "{:?}", "str": "{}", "inner": {} }}"#,
        rule.as_rule(),
        rule.as_str(),
        pairs_str(rule.into_inner()),
    )
}

pub fn pairs_str(inner: Pairs<Rule>) -> String {
    let joined_str = inner
        .map(|pair| pair_str(pair))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{joined_str}]",)
}
