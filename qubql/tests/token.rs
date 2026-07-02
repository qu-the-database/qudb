use std::assert_matches;
use qubql::{error::ResultExt, token::{Token, into_tokens}};

#[test]
fn idents_only() {
    let text = "hello hi hoi";

    let mut tokens = into_tokens(&text).into_fancy().unwrap().into_iter();

    match tokens.next().expect("token 1 is missing") {
        Token::Ident(x) => assert_eq!(x.as_str(), "hello"),
        x => panic!("Invalid token 1: {x:?}"),
    }
    match tokens.next().expect("token 2 is missing") {
        Token::Ident(x) => assert_eq!(x.as_str(), "hi"),
        x => panic!("Invalid token 2: {x:?}"),
    }
    match tokens.next().expect("token 3 is missing") {
        Token::Ident(x) => assert_eq!(x.as_str(), "hoi"),
        x => panic!("Invalid token 3: {x:?}"),
    }
    assert_matches!(tokens.next(), None);
}

#[test]
fn single_string_a() {
    let text = r#""abc""#;

    let mut tokens = into_tokens(&text).into_fancy().unwrap().into_iter();

    match tokens.next().expect("token 1 is missing") {
        Token::String(x) => assert_eq!(x.as_str(), "abc"),
        x => panic!("Invalid token 1: {x:?}"),
    }
    assert_matches!(tokens.next(), None);
}

#[test]
fn strings_only() {
    let text = r#""a" "b"'c\0'"d\0""#;

    let mut tokens = into_tokens(&text).into_fancy().unwrap().into_iter();

    match tokens.next().expect("token 1 is missing") {
        Token::String(x) => assert_eq!(x.as_str(), "a"),
        x => panic!("Invalid token 1: {x:?}"),
    }
    match tokens.next().expect("token 2 is missing") {
        Token::String(x) => assert_eq!(x.as_str(), "b"),
        x => panic!("Invalid token 2: {x:?}"),
    }
    match tokens.next().expect("token 3 is missing") {
        Token::String(x) => assert_eq!(x.as_str(), "c\\0"),
        x => panic!("Invalid token 3: {x:?}"),
    }
    match tokens.next().expect("token 4 is missing") {
        Token::String(x) => assert_eq!(x.as_str(), "d\0"),
        x => panic!("Invalid token 4: {x:?}"),
    }
    assert_matches!(tokens.next(), None);
}

#[test]
fn escapes() {
    let text = r#""a b c \n\r\0\x1b\u001b""#;

    let mut tokens = into_tokens(&text).into_fancy().unwrap().into_iter();

    match tokens.next().expect("token 1 is missing") {
        Token::String(x) => assert_eq!(x.as_str(), "a b c \n\r\0\x1b\x1b"),
        x => panic!("Invalid token 1: {x:?}"),
    }
    assert_matches!(tokens.next(), None);
}

#[test]
fn combined() {
    let text = r#"aa"bb"'cc'dd'ee'"#;

    let mut tokens = into_tokens(&text).into_fancy().unwrap().into_iter();

    match tokens.next().expect("token 1 is missing") {
        Token::Ident(x) => assert_eq!(x.as_str(), "aa"),
        x => panic!("Invalid token 1: {x:?}"),
    }
    match tokens.next().expect("token 2 is missing") {
        Token::String(x) => assert_eq!(x.as_str(), "bb"),
        x => panic!("Invalid token 2: {x:?}"),
    }
    match tokens.next().expect("token 3 is missing") {
        Token::String(x) => assert_eq!(x.as_str(), "cc"),
        x => panic!("Invalid token 3: {x:?}"),
    }
    match tokens.next().expect("token 4 is missing") {
        Token::Ident(x) => assert_eq!(x.as_str(), "dd"),
        x => panic!("Invalid token 4: {x:?}"),
    }
    match tokens.next().expect("token 5 is missing") {
        Token::String(x) => assert_eq!(x.as_str(), "ee"),
        x => panic!("Invalid token 5: {x:?}"),
    }
    assert_matches!(tokens.next(), None);
}

#[test]
fn query_test() {
    let text = r#"create table whatever fields {
                            id a: i64 default i64::rand(),
                            b: str,
                            c: bool,
                            d: uuid,
                        };
                     "#;

    let mut tokens = into_tokens(&text).into_fancy().unwrap().into_iter();

    let mut n = 0u32;

    n += 1;
    match tokens.next().expect(&format!("token {n} is missing")) {
        Token::Ident(x) => assert_eq!(x.as_str(), "create"),
        x => panic!("Invalid token {n}: {x:?}"),
    }

    n += 1;
    match tokens.next().expect(&format!("token {n} is missing")) {
        Token::Ident(x) => assert_eq!(x.as_str(), "table"),
        x => panic!("Invalid token {n}: {x:?}"),
    }

    n += 1;
    match tokens.next().expect(&format!("token {n} is missing")) {
        Token::Ident(x) => assert_eq!(x.as_str(), "whatever"),
        x => panic!("Invalid token {n}: {x:?}"),
    }

    n += 1;
    match tokens.next().expect(&format!("token {n} is missing")) {
        Token::Ident(x) => assert_eq!(x.as_str(), "fields"),
        x => panic!("Invalid token {n}: {x:?}"),
    }

    n += 1;
    match tokens.next().expect(&format!("token {n} is missing")) {
        Token::BlockCurly(x) => {
            let mut tokens = x.value.into_iter();
            
            n += 1;
            match tokens.next().expect(&format!("token {n} is missing")) {
                Token::Ident(x) => assert_eq!(x.as_str(), "id"),
                x => panic!("Invalid token {n}: {x:?}"),
            }
            
            n += 1;
            match tokens.next().expect(&format!("token {n} is missing")) {
                Token::Ident(x) => assert_eq!(x.as_str(), "a"),
                x => panic!("Invalid token {n}: {x:?}"),
            }
            
            n += 1;
            match tokens.next().expect(&format!("token {n} is missing")) {
                Token::Operator(x) => assert_eq!(x.value, ":"),
                x => panic!("Invalid token {n}: {x:?}"),
            }
            
            n += 1;
            match tokens.next().expect(&format!("token {n} is missing")) {
                Token::Ident(x) => assert_eq!(x.value, "i64"),
                x => panic!("Invalid token {n}: {x:?}"),
            }
            
            n += 1;
            match tokens.next().expect(&format!("token {n} is missing")) {
                Token::Ident(x) => assert_eq!(x.as_str(), "default"),
                x => panic!("Invalid token {n}: {x:?}"),
            }
            
            n += 1;
            match tokens.next().expect(&format!("token {n} is missing")) {
                Token::Ident(x) => assert_eq!(x.as_str(), "i64"),
                x => panic!("Invalid token {n}: {x:?}"),
            }
            
            n += 1;
            match tokens.next().expect(&format!("token {n} is missing")) {
                Token::Operator(x) => assert_eq!(x.value, "::"),
                x => panic!("Invalid token {n}: {x:?}"),
            }
            
            n += 1;
            match tokens.next().expect(&format!("token {n} is missing")) {
                Token::Ident(x) => assert_eq!(x.as_str(), "rand"),
                x => panic!("Invalid token {n}: {x:?}"),
            }
            
            n += 1;
            match tokens.next().expect(&format!("token {n} is missing")) {
                Token::BlockRound(x) => assert!(x.value.is_empty()),
                x => panic!("Invalid token {n}: {x:?}"),
            }
            
            n += 1;
            match tokens.next().expect(&format!("token {n} is missing")) {
                Token::Operator(x) => assert_eq!(x.value, ","),
                x => panic!("Invalid token {n}: {x:?}"),
            }
            
            n += 1;
            match tokens.next().expect(&format!("token {n} is missing")) {
                Token::Ident(x) => assert_eq!(x.as_str(), "b"),
                x => panic!("Invalid token {n}: {x:?}"),
            }
            
            n += 1;
            match tokens.next().expect(&format!("token {n} is missing")) {
                Token::Operator(x) => assert_eq!(x.value, ":"),
                x => panic!("Invalid token {n}: {x:?}"),
            }
            
            n += 1;
            match tokens.next().expect(&format!("token {n} is missing")) {
                Token::Ident(x) => assert_eq!(x.as_str(), "str"),
                x => panic!("Invalid token {n}: {x:?}"),
            }
            
            n += 1;
            match tokens.next().expect(&format!("token {n} is missing")) {
                Token::Operator(x) => assert_eq!(x.value, ","),
                x => panic!("Invalid token {n}: {x:?}"),
            }
            
            n += 1;
            match tokens.next().expect(&format!("token {n} is missing")) {
                Token::Ident(x) => assert_eq!(x.as_str(), "c"),
                x => panic!("Invalid token {n}: {x:?}"),
            }
            
            n += 1;
            match tokens.next().expect(&format!("token {n} is missing")) {
                Token::Operator(x) => assert_eq!(x.value, ":"),
                x => panic!("Invalid token {n}: {x:?}"),
            }
            
            n += 1;
            match tokens.next().expect(&format!("token {n} is missing")) {
                Token::Ident(x) => assert_eq!(x.as_str(), "bool"),
                x => panic!("Invalid token {n}: {x:?}"),
            }
            
            n += 1;
            match tokens.next().expect(&format!("token {n} is missing")) {
                Token::Operator(x) => assert_eq!(x.value, ","),
                x => panic!("Invalid token {n}: {x:?}"),
            }
            
            n += 1;
            match tokens.next().expect(&format!("token {n} is missing")) {
                Token::Ident(x) => assert_eq!(x.as_str(), "d"),
                x => panic!("Invalid token {n}: {x:?}"),
            }
            
            n += 1;
            match tokens.next().expect(&format!("token {n} is missing")) {
                Token::Operator(x) => assert_eq!(x.value, ":"),
                x => panic!("Invalid token {n}: {x:?}"),
            }
            
            n += 1;
            match tokens.next().expect(&format!("token {n} is missing")) {
                Token::Ident(x) => assert_eq!(x.as_str(), "uuid"),
                x => panic!("Invalid token {n}: {x:?}"),
            }
            
            n += 1;
            match tokens.next().expect(&format!("token {n} is missing")) {
                Token::Operator(x) => assert_eq!(x.value, ","),
                x => panic!("Invalid token {n}: {x:?}"),
            }

            assert_matches!(tokens.next(), None);
        },
        x => panic!("Invalid token {n}: {x:?}"),
    }

    n += 1;
    match tokens.next().expect(&format!("token {n} is missing")) {
        Token::Operator(x) => assert_eq!(x.value, ";"),
        x => panic!("Invalid token {n}: {x:?}"),
    }

    assert_matches!(tokens.next(), None);
}
