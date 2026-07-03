use crate::{error::{Error, Result}, statement::{typ::Type, utl::{next_code, peek_code}}, token::{Location, StringToken, Token}};

pub fn parse_statement<'data, 'token>(location: Location, tokens: &mut &'token [Token<'data>]) -> Result<'data, 'token, Statement<'token>> {
    let Some(token) = next_code(tokens) else {
        return Err(Error::MissingToken { loc: location, err_code: "stmt-empty" });
    };

    match token {
        Token::Ident(x) if x.as_str().eq_ignore_ascii_case("create")
            && peek_code(*tokens).is_some_and(|x| x.as_ident().is_some()) =>
                parse_create_statement(tokens),
        Token::String(x) => todo!(),
        Token::BlockCurly(x) => todo!(),
        Token::BlockRound(x) => todo!(),
        Token::BlockSquare(x) => todo!(),
        _ => todo!(),
    }
}

/// Parse a CREATE statement.
///
/// It is expected that the next token is an ident.
///
/// ```txt
/// create table whatever;
///        ^^^^^^^^^^^^^^^
///        *
/// ```
pub fn parse_create_statement<'data, 'token>(tokens: &mut &'token [Token<'data>]) -> Result<'data, 'token, Statement<'token>> {
    let token = next_code(tokens).unwrap();
    let token_i = token.as_ident().unwrap();

    match token_i.as_str() {
        x if x.eq_ignore_ascii_case("table") => parse_create_table_statement(token.location(), tokens),
        _ => Err(Error::UnexpectedToken { token, err_code: "create-unknown-kind" }),
    }
}

pub fn parse_create_table_statement<'data, 'token>(location: Location, tokens: &mut &'token [Token<'data>]) -> Result<'data, 'token, Statement<'token>> {
    let Some(name_t) = next_code(tokens) else { return Err(Error::MissingToken { loc: location, err_code: "create-table-no-name" }) };
    let Some(name) = name_t.as_ident() else { return Err(Error::MissingToken { loc: location, err_code: "create-table-noident-name" }) };

    let ty = super::typ::parse_type(location, tokens)?;

    Ok(Statement::CreateTableStatement(CreateTableStatement {
        location,
        table: name.as_str(),
        data: ty,
    }))
}

#[derive(Debug)]
pub enum Statement<'token> {
    // [value]

    ConstString(ConstStringStatement<'token>),

    // create ..

    CreateTableStatement(CreateTableStatement<'token>),
}

#[derive(Debug)]
pub struct ConstStringStatement<'token> {
    pub location: Location,
    pub token: StringToken<'token>,
}

#[derive(Debug)]
pub struct CreateTableStatement<'token> {
    pub location: Location,
    pub table: &'token str,
    pub data: Type<'token>,
}
