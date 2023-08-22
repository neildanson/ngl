use crate::parser_combinator::*;

use super::*;

#[test]
fn test_pint_1() {
    let parser = pint();
    let result = parser.parse("1234567890".into());
    let expected = Ok((
        Token {
            value: Value::Number(1234567890),
            start: 0,
            length: 10,
        },
        ContinuationState {
            remaining: "",
            position: 10,
            line_number: 0,
            line_position: 10,
        },
    ));
    assert_eq!(result, expected);
}

#[test]
fn test_pint_2() {
    let parser = pint();
    let result = parser.parse("-123".into());
    let expected = Ok((
        Token {
            value: Value::Number(-123),
            start: 0,
            length: 4,
        },
        ContinuationState {
            remaining: "",
            position: 4,
            line_number: 0,
            line_position: 4,
        },
    ));
    assert_eq!(result, expected);
}

#[test]
fn test_pquoted() {
    let parser = pquoted_string();
    let result = parser.parse("\"123\"".into());
    let expected = Ok((
        Token {
            value: Value::String("123".to_string()),
            start: 1,
            length: 3,
        },
        ContinuationState {
            remaining: "",
            position: 5,
            line_number: 0,
            line_position: 5,
        },
    ));
    assert_eq!(result, expected);
}

#[test]
fn test_param() {
    let parser = pparam();
    let result = parser.parse("left : right".into());
    let expected = Ok((
        Token {
            value: Parameter(
                Token::new("left".to_string(), 0, 4),
                Token::new("right".to_string(), 7, 5),
            ),
            start: 0,
            length: 12,
        },
        ContinuationState {
            remaining: "",
            position: 12,
            line_number: 0,
            line_position: 12,
        },
    ));
    assert_eq!(result, expected);
}

#[test]
fn test_params() {
    let parser = pparams();
    let result = parser.parse("(left : right, lefty:righty)".into());
    let expected = Ok((
        Token {
            value: vec![
                Token::new(
                    Parameter(
                        Token::new("left".to_string(), 1, 4),
                        Token::new("right".to_string(), 8, 5),
                    ),
                    1,
                    12,
                ),
                Token::new(
                    Parameter(
                        Token::new("lefty".to_string(), 15, 5),
                        Token::new("righty".to_string(), 21, 6),
                    ),
                    15,
                    12,
                ),
            ],
            start: 1,
            length: 26,
        },
        ContinuationState {
            remaining: "",
            position: 28,
            line_number: 0,
            line_position: 28,
        },
    ));
    assert_eq!(result, expected);
}

#[test]
fn test_let() {
    let parser = plet();
    let result = parser.parse("let x = 1".into());
    let expected = Ok((
        Token {
            value: ExprOrStatement::Statement(Statement::Let(
                Token::new("x".to_string(), 4, 1),
                Token::new(Expr::Value(Value::Number(1)), 8, 1),
            )),
            start: 4,
            length: 5,
        },
        ContinuationState {
            remaining: "",
            position: 9,
            line_number: 0,
            line_position: 9,
        },
    ));
    assert_eq!(result, expected);
}

#[test]
fn test_identifier_1() {
    let parser = pidentifier();
    let result = parser.parse("left".into());
    let expected = Ok((
        Token {
            value: "left".to_string(),
            start: 0,
            length: 4,
        },
        ContinuationState {
            remaining: "",
            position: 4,
            line_number: 0,
            line_position: 4,
        },
    ));
    assert_eq!(result, expected);
}

#[test]
fn test_identifier_2() {
    let parser = pidentifier();
    let result = parser.parse("left1".into());
    let expected = Ok((
        Token {
            value: "left1".to_string(),
            start: 0,
            length: 5,
        },
        ContinuationState {
            remaining: "",
            position: 5,
            line_number: 0,
            line_position: 5,
        },
    ));
    assert_eq!(result, expected);
}

#[test]
fn test_identifier_3() {
    let parser = pidentifier();
    let result = parser.parse("left_1".into());
    let expected = Ok((
        Token {
            value: "left_1".to_string(),
            start: 0,
            length: 6,
        },
        ContinuationState {
            remaining: "",
            position: 6,
            line_number: 0,
            line_position: 6,
        },
    ));
    assert_eq!(result, expected);
}

#[test]
fn test_identifier_4() {
    let parser = pidentifier();
    let result = parser.parse("_left1".into());
    let expected = Ok((
        Token {
            value: "_left1".to_string(),
            start: 0,
            length: 6,
        },
        ContinuationState {
            remaining: "",
            position: 6,
            line_number: 0,
            line_position: 6,
        },
    ));
    assert_eq!(result, expected);
}

#[test]
fn test_identifier_fail() {
    let parser = pidentifier();
    //identifiers cannot start with a number
    let result = parser.parse("1left".into());
    let expected = Err(Error::new(
        Expected::Or(
            Box::new(Expected::Or(
                Box::new(('a'..='z').into()),
                Box::new(('A'..='Z').into()),
            )),
            Box::new('_'.into()),
        ),
        "1",
        0,
        0,
        0,
    ));
    assert_eq!(result, expected);
}

#[test]
fn test_call() {
    let parser = pcall();
    let result = parser.parse("left(a,b)".into());
    let expected = Ok((
        Token {
            value: Expr::Call(
                Token::new("left".to_string(), 0, 4),
                vec![
                    Token::new(Expr::Ident("a".to_string()), 5, 1),
                    Token::new(Expr::Ident("b".to_string()), 7, 1),
                ],
            ),
            start: 0,
            length: 8,
        },
        ContinuationState {
            remaining: "",
            position: 9,
            line_number: 0,
            line_position: 9,
        },
    ));
    assert_eq!(result, expected);
}

#[test]
fn test_call_recur() {
    let parser = pcall();
    let result = parser.parse("left(a,left(b, c))".into());
    let expected = Ok((
        Token {
            value: Expr::Call(
                Token::new("left".to_string(), 0, 4),
                vec![
                    Token::new(Expr::Ident("a".to_string()), 5, 1),
                    Token::new(
                        Expr::Call(
                            Token::new("left".to_string(), 7, 4),
                            vec![
                                Token::new(Expr::Ident("b".to_string()), 12, 1),
                                Token::new(Expr::Ident("c".to_string()), 15, 1),
                            ],
                        ),
                        7,
                        9,
                    ),
                ],
            ),
            start: 0,
            length: 16,
        },
        ContinuationState {
            remaining: "",
            position: 18,
            line_number: 0,
            line_position: 18,
        },
    ));
    assert_eq!(result, expected);
}
