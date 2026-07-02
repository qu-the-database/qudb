pub mod abs;
pub mod stm;
pub mod typ;
pub mod utl;

use crate::{error::{Error, Result}, statement::stm::Statement, token::Token};

// pub fn into_statements<'data, I: IntoIterator<Item = Token<'data>>>(iter: I) -> IntoStatements<'data, I::IntoIter> {
//     IntoStatements { tokens: OnlyCode(iter.into_iter()).peekable() }
// }
pub fn into_statements<'data, 'borrow>(mut tokens: &'borrow [Token<'data>]) -> Result<'data, 'borrow, Vec<Statement<'borrow>>> {
    // This could've been smarter, but good enough for now.
    let mut res = Vec::with_capacity(tokens.iter()
        .filter(|x| x.as_op().is_some_and(|x| x.value == ";")).count() + 1);
    let mut errors = Vec::new();

    while !tokens.is_empty() {
        let mut tokens = match tokens.iter().enumerate()
            .find(|(_, x)| x.as_op().is_some_and(|x| x.value == ";"))
            .map(|(i, _)| i) {
                Some(i) => {
                    let (left, right) = tokens.split_at(i);
                    tokens = &right[1..];
                    left
                }
                None => {
                    tokens
                }
            };

        if tokens.is_empty() { continue; }

        match stm::parse_statement(tokens[0].location(), &mut tokens) {
            Ok(x) => res.push(x),
            Err(why) => {
                let r = why.is_recoverable();
                errors.push(why);
                if !r { break }
            },
        }
    }

    if errors.is_empty() {
        Ok(res)
    } else if errors.len() == 1 {
        Err(errors.remove(0))
    } else {
        Err(Error::MultipleErrors(errors))
    }
}

// pub struct OnlyCode<'data, I: Iterator<Item = Token<'data>>>(I);
// impl<'data, I: Iterator<Item = Token<'data>>> Iterator for OnlyCode<'data, I> {
//     type Item = Token<'data>;
// 
//     fn next(&mut self) -> Option<Self::Item> {
//         while let Some(x) = self.0.next() {
//             if x.importance() < TokenImportance::Docs { continue }
//             return Some(x);
//         }
//         None
//     }
// }
// 
// type InnerIt<'data, I> = Peekable<OnlyCode<'data, I>>;
// 
// pub struct IntoStatements<'data, I: Iterator<Item = Token<'data>>> {
//     tokens: InnerIt<'data, I>,
// }
// impl<'data, I: Iterator<Item = Token<'data>>> IntoStatements<'data, I> {
//     pub fn finish_parsing(self) -> Result<'data, Vec<Statement<'data>>> {
//         enum Storing<'a> {
//             Statements(Vec<Statement<'a>>),
//             Errors(Vec<Error<'a>>),
//         }
//         let mut ret = Storing::Statements(Vec::new());
// 
//         for x in self {
//             match x {
//                 Ok(st) => if let Storing::Statements(ret) = &mut ret {
//                     ret.push(st);
//                 },
//                 Err(why) => match &mut ret {
//                     Storing::Statements(_) => ret = Storing::Errors(vec![why]),
//                     Storing::Errors(errors) => errors.push(why),
//                 },
//             };
//         }
// 
//         match ret {
//             Storing::Statements(statements) => Ok(statements),
//             Storing::Errors(mut errors) => if errors.len() == 1 {
//                 Err(errors.pop().unwrap())
//             } else {
//                 Err(Error::MultipleErrors(errors))
//             },
//         }
//     }
// }
// impl<'data, I: Iterator<Item = Token<'data>>> Iterator for IntoStatements<'data, I> {
//     type Item = Result<'data, Statement<'data>>;
// 
//     fn next(&mut self) -> Option<Self::Item> { 'fnstart: loop {
//         let Some(root) = self.tokens.next() else {
//             return None;
//         };
// 
//         match root.kind {
//             TokenKind::Operator(";") => continue 'fnstart,
//             TokenKind::Comment(_) => unreachable!(),
//             TokenKind::Doc(_) => todo!(),
//             TokenKind::Ident(_) => {
//                 let r = parse_statement(&mut Carriage::new(root), &mut self.tokens);
//                 if r.as_ref().is_err_and(|x| x.is_recoverable()) {
//                     while self.tokens.peek().is_some_and(|x| !matches!(x.kind, TokenKind::Operator(";"))) {
//                         self.tokens.next();
//                     }
//                 }
//                 return Some(r);
//             },
//             _ => return Some(Err(Error::StatementInvalidToken { token: root, err_code: "stmt-nonident-start" })),
//         }
//     } }
// }

// macro_rules! tident {
//     ($expr:expr) => {
//         match $expr {
//             TokenKind::Ident(x) => x,
//             _ => unreachable!(),
//         }
//     };
// }
// 
// macro_rules! deref_enum {
//     ($expr:expr, $($var:tt)*) => {
//         match $expr {
//             $($var)*,
//             _ => unreachable!(),
//         }
//     };
// }
// 
// struct Carriage<T>(Option<T>);
// impl<T> Carriage<T> {
//     pub const fn new(value: T) -> Self {
//         Self(Some(value))
//     }
// 
//     pub fn into_inner(self) -> T {
//         self.0.expect("value was moved out")
//     }
// 
//     pub fn take_inner(&mut self) -> T {
//         self.0.take().expect("value was moved out")
//     }
// 
//     pub const fn taken(&mut self) -> bool {
//         self.0.is_none()
//     }
// 
//     pub const fn take_checked(&mut self) -> Option<T> {
//         self.0.take()
//     }
// }
// impl<T> AsRef<T> for Carriage<T> {
//     fn as_ref(&self) -> &T {
//         self.0.as_ref().expect("value was moved out")
//     }
// }
// impl<T> AsMut<T> for Carriage<T> {
//     fn as_mut(&mut self) -> &mut T {
//         self.0.as_mut().expect("value was moved out")
//     }
// }
// impl<T> ops::Deref for Carriage<T> {
//     type Target = T;
// 
//     fn deref(&self) -> &Self::Target {
//         self.as_ref()
//     }
// }
// impl<T> ops::DerefMut for Carriage<T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         self.as_mut()
//     }
// }
// 
// /// Some of the starting idents that will result in a value statement being parsed.
// ///
// /// All strings, numbers, and idents starting with "$" will count too.
// static VALUE_STATEMENT_HEADS: &[&str] = &[
//     "false",
//     "true",
//     "uuid",
//     "bool",   "boolean",  "bit",
//     "array",
//     "object", "document", "fields",
//     "record",
//     "string", "str",
//     "i8",    
//     "u8",     "byte",
//     "i16",    "short",
//     "u16",    "ushort",
//     "i32",    "int",
//     "u32",    "uint",
//     "i64",    "long",
//     "u64",    "ulong",
//     "f32",    "float",
//     "f64",    "double",
//     "fn",
//     "mod",
// ];
// 
// fn parse_statement<'data, I: Iterator<Item = Token<'data>>>(root: &mut Carriage<Token<'data>>, iter: &mut InnerIt<'data, I>) -> Result<'data, Statement<'data>> {
//     let ident = tident!(&root.kind);
//     if ident.eq_ignore_ascii_case("create") {
//         return parse_create_statement(root, iter);
//     } else {
//         return Err(Error::StatementInvalidToken { token: root.take_inner(), err_code: "stmt-inval-start" });
//     }
// }
// 
// fn parse_create_statement<'data, I: Iterator<Item = Token<'data>>>(root: &mut Carriage<Token<'data>>, iter: &mut InnerIt<'data, I>) -> Result<'data, Statement<'data>> {
//     let Some(kind) = iter.next() else {
//         return Err(Error::MissingToken { loc: root.location, err_code: "stmt-create-nokind" });
//     };
// 
//     let ident = match kind.kind {
//         TokenKind::Ident(x) => x,
//         _ => return Err(Error::UnexpectedToken { token: kind, err_code: "stmt-create-nonidentkind" }),
//     };
// 
//     if ident.eq_ignore_ascii_case("table") {
//         return parse_create_table_statement(root, iter);
//     } else {
//         return Err(Error::InvalidObjectKind { provided: ident, loc: kind.location, err_code: "stmt-create-invalkind" });
//     }
// }
// 
// fn parse_create_table_statement<'data, I: Iterator<Item = Token<'data>>>(
//     root: &mut Carriage<Token<'data>>,
//     iter: &mut InnerIt<'data, I>
// ) -> Result<'data, Statement<'data>> {
//     let Some(table) = iter.next() else {
//         return Err(Error::MissingToken { loc: root.location, err_code: "stmt-create-notable" });
//     };
// 
//     let table_str = match table.kind {
//         TokenKind::Ident(x) => x,
//         _ => return Err(Error::UnexpectedToken { token: table, err_code: "stmt-create-nonidenttable" }),
//     };
// 
//     // let Some(data) = iter.next() else {
//     //     return Err(Error::MissingToken { loc: root.location, err_code: "stmt-create-nodata" });
//     // };
// 
//     // let data_str = match data.kind {
//     //     TokenKind::Ident(x) => x,
//     //     _ => return Err(Error::UnexpectedToken { token: data, err_code: "stmt-create-nonidentdata" }),
//     // };
// 
//     // Ok(Statement {
//     //     loc: root.location,
//     //     kind: StatementKind::Create(CreateStatement::CreateTableStatement(CreateTableStatement {
//     //         table: table_str,
//     //         data: if data_str.eq_ignore_ascii_case("fields") {
//     //             Type::Fields(parse_fields(root, iter)?)
//     //         } else {
//     //             return Err(Error::InvalidObjectKind { provided: data_str, loc: data.location, err_code: "stmt-create-invaldata" })
//     //         }
//     //     }))
//     // })
// 
//     Ok(Statement {
//         loc: root.location,
//         kind: StatementKind::Create(CreateStatement::CreateTableStatement(CreateTableStatement {
//             table: table_str,
//             data: parse_type(root, iter)?,
//         }))
//     })
// }

// #[derive(Debug)]
// pub struct Statement<'data> {
//     pub loc: Location,
//     pub kind: StatementKind<'data>,
// }
// 
// #[derive(Debug)]
// pub enum StatementKind<'data> {
//     Create(CreateStatement<'data>),
// }
// 
// #[derive(Debug)]
// pub enum CreateStatement<'data> {
//     CreateTableStatement(CreateTableStatement<'data>),
// }
// 
// #[derive(Debug)]
// pub struct CreateTableStatement<'data> {
//     /// Name of the table.
//     pub table: Str<'data>,
//     /// Data stored in the table.
//     pub data: Type<'data>,
// }
