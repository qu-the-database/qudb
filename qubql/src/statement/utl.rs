use crate::token::Token;

#[inline(always)]
pub fn peek_code<'data, 'token>(mut tokens: &'token [Token<'data>]) -> Option<&'token Token<'data>> {
    return next_code(&mut tokens);
}

pub fn next_code<'data, 'token>(tokens: &mut &'token [Token<'data>]) -> Option<&'token Token<'data>> {
    while let Some(x) = tokens.get(0) {
        *tokens = &tokens[1..];
        if x.importance().is_code() {
            return Some(x);
        }
    }

    None
}
