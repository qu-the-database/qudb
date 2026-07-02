use crate::token::Token;

/// Find all potential statements.
///
/// A statement ends either with a `;` or an EOF.
pub fn into_intermediate<'data, 'tokens>(tokens: &'tokens [Token<'data>]) -> Vec<&'tokens [Token<'data>]> {
    if tokens.is_empty() { return Vec::new() }

    let mut acc = Vec::with_capacity(1 + tokens.iter().filter(|x| x.as_op().is_some_and(|x| x.value == ";")).count());

    let mut i = 0usize;
    for (o, token) in tokens.iter().enumerate() {
        if token.as_op().is_some_and(|x| x.value == ";") {
            if i - o > 1 {
                acc.push(&tokens[i..o]);
            }
            i = o;
        }
    }

    if tokens.len() - i > 1 {
        acc.push(&tokens[i..]);
    }

    acc
}
