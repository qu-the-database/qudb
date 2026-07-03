use qubql::{error::ResultExt, statement::{into_statements, stm::Statement, typ::Type}, token::into_tokens};
use std::assert_matches;

#[test]
fn basic_create() {
    let text = r#"create table whatever fields {
                            a: i64,
                            b: str,
                            c: bool,
                            d: uuid,
                        };
                     "#;
    let tokens = into_tokens(text).into_fancy().unwrap();
    let statements = into_statements(&tokens).into_fancy().unwrap();

    assert!(statements.len() == 1);
    match &statements.get(0).unwrap() {
        Statement::CreateTableStatement(x) => {
            assert_eq!(x.table, "whatever");
            assert_matches!(x.data, Type::Fields(_));
            match &x.data {
                Type::Fields(x) => {
                    assert_eq!(x.fields.len(), 4);

                    let field = x.fields.get("a").unwrap();
                    assert_eq!(field.id, false);
                    assert_matches!(field.def, None);
                    assert_matches!(field.ty, Type::I64);

                    let field = x.fields.get("b").unwrap();
                    assert_eq!(field.id, false);
                    assert_matches!(field.def, None);
                    assert_matches!(field.ty, Type::Str);

                    let field = x.fields.get("c").unwrap();
                    assert_eq!(field.id, false);
                    assert_matches!(field.def, None);
                    assert_matches!(field.ty, Type::Bool);

                    let field = x.fields.get("d").unwrap();
                    assert_eq!(field.id, false);
                    assert_matches!(field.def, None);
                    assert_matches!(field.ty, Type::Uuid);
                }
                _ => unreachable!(),
            }
        },
        x => panic!("wrong statement! {x:?}"),
    }
}

#[test]
fn create_with_field_params() {
    let text = r#"create table whatever fields {
                            a: i64 id,
                            b: str,
                            c: bool,
                            d: uuid,
                        };
                       "#;
    let tokens = into_tokens(text).into_fancy().unwrap();
    let statements = into_statements(&tokens).into_fancy().unwrap();

    assert!(statements.len() == 1);
    match &statements.get(0).unwrap() {
        Statement::CreateTableStatement(x) => {
            assert_eq!(x.table, "whatever");
            assert_matches!(x.data, Type::Fields(_));
            match &x.data {
                Type::Fields(x) => {
                    assert_eq!(x.fields.len(), 4);

                    let field = x.fields.get("a").unwrap();
                    assert_eq!(field.id, true);
                    assert_matches!(field.def, None);
                    assert_matches!(field.ty, Type::I64);

                    let field = x.fields.get("b").unwrap();
                    assert_eq!(field.id, false);
                    assert_matches!(field.def, None);
                    assert_matches!(field.ty, Type::Str);

                    let field = x.fields.get("c").unwrap();
                    assert_eq!(field.id, false);
                    assert_matches!(field.def, None);
                    assert_matches!(field.ty, Type::Bool);

                    let field = x.fields.get("d").unwrap();
                    assert_eq!(field.id, false);
                    assert_matches!(field.def, None);
                    assert_matches!(field.ty, Type::Uuid);
                }
                _ => unreachable!(),
            }
        },
        x => panic!("wrong statement! {x:?}"),
    }
}

#[test]
fn create_with_field_default() {
    let text = r#"create table whatever fields {
                            a: i64 id,
                            b: str default { "Hello!" },
                            c: bool default { false },
                            d: uuid default { uuid::rand() },
                        };
                       "#;
    let tokens = into_tokens(text).into_fancy().unwrap();
    let statements = into_statements(&tokens).into_fancy().unwrap();

    assert!(statements.len() == 1);
    match &statements.get(0).unwrap() {
        Statement::CreateTableStatement(x) => {
            assert_eq!(x.table, "whatever");
            assert_matches!(x.data, Type::Fields(_));
            match &x.data {
                Type::Fields(x) => {
                    assert_eq!(x.fields.len(), 4);

                    let field = x.fields.get("a").unwrap();
                    assert_eq!(field.id, true);
                    assert_matches!(field.def, None);
                    assert_matches!(field.ty, Type::I64);

                    let field = x.fields.get("b").unwrap();
                    assert_eq!(field.id, false);
                    assert_matches!(field.def, None);
                    assert_matches!(field.ty, Type::Str);

                    let field = x.fields.get("c").unwrap();
                    assert_eq!(field.id, false);
                    assert_matches!(field.def, None);
                    assert_matches!(field.ty, Type::Bool);

                    let field = x.fields.get("d").unwrap();
                    assert_eq!(field.id, false);
                    assert_matches!(field.def, None);
                    assert_matches!(field.ty, Type::Uuid);
                }
                _ => unreachable!(),
            }
        },
        x => panic!("wrong statement! {x:?}"),
    }
}

#[test]
fn create_with_nested_objects() {
    let text = r#"create table whatever fields {
                            a: fields {
                                b: object {
                                    c: {
                                        d: object,
                                    },
                                },
                            },
                        };
                     "#;
    let tokens = into_tokens(text).into_fancy().unwrap();
    let statements = into_statements(&tokens).into_fancy().unwrap();

    assert!(statements.len() == 1);
    match &statements.get(0).unwrap() {
        Statement::CreateTableStatement(x) => {
            assert_eq!(x.table, "whatever");
            assert_matches!(x.data, Type::Fields(_));
            match &x.data {
                Type::Fields(x) => {
                    assert_eq!(x.fields.len(), 1);

                    let field = x.fields.get("a").unwrap();
                    assert_eq!(field.id, false);
                    assert_matches!(field.def, None);
                    match &field.ty {
                        Type::Fields(x) => {
                            assert_eq!(x.fields.len(), 1);
                            let field = x.fields.get("b").unwrap();
                            assert_eq!(field.id, false);
                            assert_matches!(field.def, None);
                            match &field.ty {
                                Type::Fields(x) => {
                                    assert_eq!(x.fields.len(), 1);
                                    let field = x.fields.get("c").unwrap();
                                    assert_eq!(field.id, false);
                                    assert_matches!(field.def, None);
                                    match &field.ty {
                                        Type::Fields(x) => {
                                            assert_eq!(x.fields.len(), 1);
                                            let field = x.fields.get("d").unwrap();
                                            assert_eq!(field.id, false);
                                            assert_matches!(field.def, None);
                                            assert_matches!(field.ty, Type::Document);
                                        }
                                        x => panic!("expected fields (nesting 3), found {x:?}"),
                                    }
                                }
                                x => panic!("expected fields (nesting 2), found {x:?}"),
                            }
                        }
                        x => panic!("expected fields (nesting 1), found {x:?}"),
                    }
                }
                _ => unreachable!(),
            }
        },
        x => panic!("wrong statement! {x:?}"),
    }
}

#[test]
fn create_document() {
    let text = r#"create table whatever document"#;
    let tokens = into_tokens(text).into_fancy().unwrap();
    let statements = into_statements(&tokens).into_fancy().unwrap();

    assert!(statements.len() == 1);
    match &statements.get(0).unwrap() {
        Statement::CreateTableStatement(x) => {
            assert_eq!(x.table, "whatever");
            assert_matches!(x.data, Type::Document);
        },
        x => panic!("wrong statement! {x:?}"),
    }
}
