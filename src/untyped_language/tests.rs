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
