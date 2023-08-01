use crate::parser_combinator::*;

use super::*;

#[test]
fn test_pint_1() {
    let parser = pint();
    let result = parser.parse("123".into());
    let expected = Ok((
        Token {
            value: Value::Number(123),
            start: 0,
            length: 3,
        },
        ContinuationState {
            remaining: "",
            position: 3,
            line_number: 0,
            line_position: 3,
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
        "A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z, a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p, q, r, s, t, u, v, w, x, y, z or _".to_string(),
        "1".to_string(),
        0,
        0,
        0,
    ));
    assert_eq!(result, expected);
}
