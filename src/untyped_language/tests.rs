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
            value: Parameter("left".to_string(), "right".to_string()),
            start: 0,
            length: 9,
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
                Token::new(Parameter("left".to_string(), "right".to_string()), 1, 9),
                Token::new(Parameter("lefty".to_string(), "righty".to_string()), 15, 11),
            ],
            start: 1,
            length: 12,
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
