use std::{collections::HashMap, fmt};

use crate::{error::{Error, Result}, statement::{into_statements, stm::Statement, utl::{next_code, peek_code}}, token::{BlockCurlyToken, Location, Token}};

/// Parse fields.
///
/// ```txt
/// fields {
///        ^
///        x
///     field1: ty id,
///     ^^^^^^^^^^^^^^
///     field2: ty id,
///     ^^^^^^^^^^^^^^
/// }
/// ^
/// ```
fn parse_fields<'data, 'token>(token: &'token BlockCurlyToken<'data>)
    -> Result<'data, 'token, Fields<'token>>
where 'data: 'token
{
    let mut res = HashMap::new();
    let mut errors = Vec::new();
    let mut tokens = token.value.as_slice();
    let mut location = Location::ZERO;

    while !tokens.is_empty() {
        let end = tokens
            .into_iter()
            .enumerate()
            .find(|(_, x)| x.as_op().is_some_and(|x| x.value == ","))
            .map(|x| x.0);

        let mut s = match end {
            Some(i) => {
                let s = &tokens[..i];
                location = tokens[i].location();
                tokens = &tokens[i + 1..];
                s
            }
            None => {
                std::mem::replace(&mut tokens, &[])
            },
        };
        
        if !s.is_empty() {
            let start_t = &s[0];
            match parse_field(location, &mut s) {
                Ok((name, field)) => {
                    if let Some((_, prev_loc)) = res.insert(name, (field, start_t.location())) {
                        errors.push(Error::DuplicateField { prev_loc, token: start_t, err_code: "fields-dup" });
                    }
                }
                Err(why) => {
                    let rec = why.is_recoverable();
                    errors.push(why);
                    if !rec { break }
                }
            }
        }
    }
    // while let Some(name) = fields_block.next() {
    //     let mut name = Carriage::new(name);
    //     match parse_field(&mut name, &mut fields_block) {
    //         Ok(None) => continue,
    //         Ok(Some(x)) => {
    //             if res.contains_key(x.0.as_ref()) {
    //                 errors.push(Error::DuplicateField { token: name.take_inner(), err_code: "fields-dup" });
    //                 continue;
    //             }

    //             #[cfg(debug_assertions)]
    //             assert!(res.insert(x.0, x.1).is_none());
    //         }
    //         Err(why) => {
    //             if why.is_recoverable() {
    //                 while fields_block.peek().is_some_and(|x| !matches!(x.kind, TokenKind::Operator(","))) {
    //                     fields_block.next();
    //                 }
    //             }
    //             errors.push(why);
    //         }
    //     }
    // }

    if errors.is_empty() {
        Ok(Fields { fields: HashMap::from_iter(res.into_iter().map(|(k, (v, _))| (k, v))) })
    } else if errors.len() == 1 {
        Err(errors.pop().unwrap())
    } else {
        Err(Error::MultipleErrors(errors))
    }
}

/// Parse a field.
///
/// ```txt
/// {
///     field1: ty id,
///     ^^^^^^^^^^^^^^
///     x
///     field2: ty id,
/// }
/// ```
fn parse_field<'data, 'token>(location: Location, tokens: &mut &'token [Token<'data>])
    -> Result<'data, 'token, (&'token str, Field<'token>)>
where 'data: 'token
{
    let Some(name) = next_code(tokens) else {
        return Err(Error::MissingToken { loc: location, err_code: "fields-no-name" })
    };
    let Some(name) = name.as_ident() else {
        return Err(Error::UnexpectedToken { token: name, err_code: "fields-noident-name" })
    };

    let Some(sep) = next_code(tokens) else {
        return Err(Error::MissingToken { loc: location, err_code: "fields-no-sep" })
    };
    let Some(sep_o) = sep.as_op() else {
        return Err(Error::UnexpectedToken { token: sep, err_code: "fields-noop-sep" })
    };
    if sep_o.value != ":" {
        return Err(Error::UnexpectedToken { token: sep, err_code: "fields-inval-sep" })
    };

    let ty = parse_type(sep.location(), tokens)?;

    let mut id = false;
    let mut def = None;

    while let Some(token) = next_code(tokens) {
        if token.as_ident().is_some_and(|x| x.value == "id") {
            if id {
                return Err(Error::UnexpectedToken { token, err_code: "fields-id-dup" })
            } else {
                id = true;
            }
        } else if token.as_ident().is_some_and(|x| x.value == "default") {
            if def.is_some() {
                return Err(Error::UnexpectedToken { token, err_code: "fields-default-dup" })
            } else {
                let Some(next_token) = next_code(tokens) else {
                    return Err(Error::MissingToken { loc: token.location(), err_code: "fields-default-noblock" })
                };
                let Some(next_token_b) = next_token.as_curly_block() else {
                    return Err(Error::MissingToken { loc: token.location(), err_code: "fields-default-noblock" })
                };

                let content = into_statements(&next_token_b.value)?;

                def = Some(QueryBlock { content });
            }
        }
    }

    Ok((name.value.as_ref(), Field {
        id,
        def,
        ty,
    }))
}

/// Parse a type.
///
/// ```txt
/// a: i32
///    ^^^
///    x
/// ```
pub fn parse_type<'data, 'token>(location: Location, section: &mut &'token [Token<'data>]) -> Result<'data, 'token, Type<'token>> {
    let Some(typename) = next_code(section) else {
        return Err(Error::MissingToken { loc: location, err_code: "missing-type" });
    };

    let mut ty = if let Some(x) = typename.as_curly_block() {
        Type::Fields(parse_fields(x)?)
    } else if let Some(typename_s) = typename.as_ident() {
        match typename_s.as_str() {
            "i8" => Type::I8,
            "u8" | "byte" => Type::U8,
            "u16" | "ushort" => Type::U16,
            "i16" | "short" => Type::I16,
            "u32" | "uint" => Type::U32,
            "i32" | "int" => Type::I32,
            "u64" | "ulong" => Type::U64,
            "i64" | "long" => Type::I64,
            "f32" | "float" => Type::F32,
            "f64" | "double" => Type::F64,
            "string" | "str" => Type::Str,
            "bool" | "boolean" | "bit" => Type::Bool,
            "fields" => {
                let Some(token) = next_code(section) else {
                    return Err(Error::MissingToken { loc: typename.location(), err_code: "no-fields-token" });
                };
                let Some(token) = token.as_curly_block() else {
                    return Err(Error::UnexpectedToken { token: typename, err_code: "inval-fields-token" });
                };
                Type::Fields(parse_fields(token)?)
            },
            "document" => Type::Document,
            "object" => {
                if let Some(x) = peek_code(*section).and_then(|x| x.as_curly_block()) {
                    next_code(section);
                    Type::Fields(parse_fields(x)?)
                } else {
                    Type::Document
                }
            }
            "uuid" => Type::Uuid,
            "record" => todo!(),
            _ => return Err(Error::InvalidObjectKind { provided: typename_s.value.borrowed(), loc: typename.location(), err_code: "type-inval" }),
        }
    } else {
        return Err(Error::UnexpectedToken { token: typename, err_code: "type-inval-token" });
    };

    while !section.is_empty() {
        let token = &section[0];
        if token.as_square_block().is_some_and(|x| x.value.is_empty()) {
            *section = &section[1..];
            ty = Type::Array(Box::new(ty));
        }
        else if token.as_op().is_some_and(|x| x.value == "?") {
            *section = &section[1..];
            ty = Type::Option(Box::new(ty));
        }
        else { break }
    }

    Ok(ty)
}

#[derive(Debug)]
pub enum Type<'token> {
    Fields(Fields<'token>),
    Document,
    Record(&'token str),
    Array(Box<Type<'token>>),
    Option(Box<Type<'token>>),
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    U64,
    F32,
    F64,
    Str,
    Bool,
    Uuid,
}
impl Type<'_> {
    // There's probably a better way of doing this, but idk.
    fn fmt(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        for _ in 0..indent {
            f.write_str("    ")?;
        }
        match self {
            Type::Fields(fields) => {
                f.write_str("fields {")?;
                for (k, v) in &fields.fields {
                    f.write_fmt(format_args!("{k}: "))?;
                    v.ty.fmt(f, indent + 1)?;
                    if v.id { f.write_str(" id")?; }
                    // TODO: Defaults.
                }
                f.write_str(",")?;
                for _ in 0..indent {
                    f.write_str("    ")?;
                }
                f.write_str("}")?;
                Ok(())
            },
            Type::Document => f.write_str("document"),
            Type::Record(x) => f.write_fmt(format_args!("record<{x}>")),
            Type::Array(x) => {
                x.fmt(f, indent)?;
                f.write_str("[]")
            },
            Type::Option(x) => {
                x.fmt(f, indent)?;
                f.write_str("?")
            },
            Type::I8 => f.write_str("i8"),
            Type::U8 => f.write_str("u8"),
            Type::I16 => f.write_str("i16"),
            Type::U16 => f.write_str("u16"),
            Type::I32 => f.write_str("i32"),
            Type::U32 => f.write_str("u32"),
            Type::I64 => f.write_str("i64"),
            Type::U64 => f.write_str("u64"),
            Type::F32 => f.write_str("f32"),
            Type::F64 => f.write_str("f64"),
            Type::Str => f.write_str("str"),
            Type::Bool => f.write_str("bool"),
            Type::Uuid => f.write_str("uuid"),
        }
    }
}
impl fmt::Display for Type<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.fmt(f, 0)
    }
}

#[derive(Debug)]
pub struct QueryBlock<'token> {
    pub content: Vec<Statement<'token>>,
}

/// A defined object type.
#[derive(Debug)]
pub struct Fields<'token> {
    pub fields: HashMap<&'token str, Field<'token>>,
}

#[derive(Debug)]
pub struct Field<'token> {
    /// Type.
    pub ty: Type<'token>,
    /// Whether this field is used as an identifier.
    ///
    /// Multiple ids for a complex id.
    ///
    /// A `fields` table must have at least one id.
    pub id: bool,
    /// Default value.
    pub def: Option<QueryBlock<'token>>,
}
