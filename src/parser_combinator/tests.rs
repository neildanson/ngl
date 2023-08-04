use super::*;
#[test]
fn test_pchar_eof() {
    let parser = pchar('H');
    let result = parser.parse("".into());
    let expected = Err(Error::new("H".to_string(), "".to_string(), 0, 0, 0));
    assert_eq!(result, expected);
}

#[test]
fn test_pchar_wrong_letter() {
    let parser = pchar('H');
    let result = parser.parse("c".into());
    let expected = Err(Error::new("H".to_string(), "c".to_string(), 0, 0, 0));
    assert_eq!(result, expected);
}

#[test]
fn test_pchar_success() {
    let parser = pchar('H');
    let result = parser.parse("H".into());
    let expected = Ok((
        Token {
            value: 'H',
            start: 0,
            length: 1,
        },
        ContinuationState {
            remaining: "",
            position: 1,
            line_number: 0,
            line_position: 1,
        },
    ));
    assert_eq!(result, expected);
}

#[test]
fn test_pthen_success_1() {
    let parser = pthen(pchar('H'), pchar('e'));
    let result = parser.parse("Hello".into());
    let expected = Ok((
        Token {
            value: (Token::new('H', 0, 1), Token::new('e', 1, 1)),
            start: 0,
            length: 2,
        },
        ContinuationState {
            remaining: "llo",
            position: 2,
            line_number: 0,
            line_position: 2,
        },
    ));
    assert_eq!(result, expected);
}

#[test]
fn test_pthen_success_2() {
    let parser = pthen(pchar('H'), pchar('e'));
    let result = parser.parse("He".into());
    let expected = Ok((
        Token {
            value: (Token::new('H', 0, 1), Token::new('e', 1, 1)),
            start: 0,
            length: 2,
        },
        ContinuationState {
            remaining: "",
            position: 2,
            line_number: 0,
            line_position: 2,
        },
    ));
    assert_eq!(result, expected);
}

#[test]
fn test_por_success_1() {
    let parser = por(pchar('H'), pchar('h'));
    let result = parser.parse("H".into());
    let expected = Ok((
        Token {
            value: 'H',
            start: 0,
            length: 1,
        },
        ContinuationState {
            remaining: "",
            position: 1,
            line_number: 0,
            line_position: 1,
        },
    ));
    assert_eq!(result, expected);
}

#[test]
fn test_por_success_2() {
    let parser = por(pchar('H'), pchar('h'));
    let result = parser.parse("h".into());
    let expected = Ok((
        Token {
            value: 'h',
            start: 0,
            length: 1,
        },
        ContinuationState {
            remaining: "",
            position: 1,
            line_number: 0,
            line_position: 1,
        },
    ));
    assert_eq!(result, expected);
}

#[test]
fn test_por_success_fail() {
    let parser = por(pchar('H'), pchar('h'));
    let result = parser.parse("e".into());
    let expected = Err(Error::new("H or h".to_string(), "e".to_string(), 0, 0, 0));
    assert_eq!(result, expected);
}

#[test]
fn test_pmap_success() {
    let parser = pmap(pchar('T'), |_| true);
    let result = parser.parse("T".into());
    let expected = Ok((
        Token {
            value: true,
            start: 0,
            length: 1,
        },
        ContinuationState {
            remaining: "",
            position: 1,
            line_number: 0,
            line_position: 1,
        },
    ));
    assert_eq!(result, expected);
}

#[test]
fn test_poptional_success() {
    let parser = poptional(pchar('T'));
    let result: ParseResult<Option<char>> = parser.parse("T".into());
    let expected = Ok((
        Token {
            value: Some('T'),
            start: 0,
            length: 1,
        },
        ContinuationState {
            remaining: "",
            position: 1,
            line_number: 0,
            line_position: 1,
        },
    ));
    assert_eq!(result, expected);
}

#[test]
fn test_poptional_success_with_failure() {
    let parser = poptional(pchar('h'));
    let result = parser.parse("T".into());
    let expected: ParseResult<Option<char>> = Ok((
        Token {
            value: None,
            start: 0,
            length: 0,
        },
        ContinuationState {
            remaining: "T",
            position: 0,
            line_number: 0,
            line_position: 0,
        },
    ));
    assert_eq!(result, expected);
}

#[test]

fn test_pstring_eof() {
    let h_parser = pstring("Hello");
    let result = h_parser.parse("Hell".into());
    let expected = Err(Error::new("Hello".to_string(), "Hell".to_string(), 4, 0, 4));
    assert_eq!(result, expected);
}

#[test]
fn test_pstring_wrong_letter() {
    let h_parser = pstring("Hello");
    let result = h_parser.parse("c".into());
    let expected = Err(Error::new("Hello".to_string(), "c".to_string(), 0, 0, 0));
    assert_eq!(result, expected);
}

#[test]
fn test_pstring_wrong_letter_after_other_parse() {
    let parser1 = pthen(pchar('c'), pchar('w'));
    let parser = pthen(parser1, pstring("Hello"));
    let result = parser.parse("cwrong".into());
    let expected = Err(Error::new("Hello".to_string(), "r".to_string(), 2, 0, 2));
    assert_eq!(result, expected);
}

#[test]
fn test_pstring_success() {
    let h_parser = pstring("Hello");
    let result = h_parser.parse("Hello".into());
    let expected = Ok((
        Token {
            value: "Hello",
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
fn test_pchar_followed_by_pstring_followed_by_failure() {
    let parser1 = pthen(pchar('c'), pstring("Hello"));
    let parser = pthen(parser1, pchar('w'));
    let result = parser.parse("cHelloX".into());
    let expected = Err(Error::new("w".to_string(), "X".to_string(), 6, 0, 6));
    assert_eq!(result, expected);
}

#[test]
fn test_correct_line_number_on_error() {
    let parser = pthen(pchar('\n'), pchar('\n'));
    let parser = pthen(parser, pchar('a'));
    let result = parser.parse("\n\nb".into());
    let expected = Err(Error::new("a".to_string(), "b".to_string(), 2, 2, 0));
    assert_eq!(result, expected);
}

#[test]
fn test_pchoice_success() {
    let parser = pchoice(vec![pchar('a'), pchar('b')]);
    let result = parser.parse("a".into());
    let expected = Ok((
        Token {
            value: 'a',
            start: 0,
            length: 1,
        },
        ContinuationState {
            remaining: "",
            position: 1,
            line_number: 0,
            line_position: 1,
        },
    ));
    assert_eq!(result, expected);
}

#[test]
fn test_pchoice_fail() {
    let parser = pchoice(vec![pchar('a'), pchar('b')]);
    let result = parser.parse("c".into());
    let expected = Err(Error::new("a or b".to_string(), "c".to_string(), 0, 0, 0));
    assert_eq!(result, expected);
}

#[test]
fn test_pchoice_fail_macro() {
    let parser = pchoice!(pchar('a'), pchar('b'), pchar('c'));
    let result = parser.parse("d".into());
    let expected = Err(Error::new(
        "a or b or c".to_string(),
        "d".to_string(),
        0,
        0,
        0,
    ));
    assert_eq!(result, expected);
}

#[test]
fn test_pany_success() {
    let parser = pany(&['a', 'b', 'c']);
    let result = parser.parse("b".into());
    let expected = Ok((
        Token {
            value: 'b',
            start: 0,
            length: 1,
        },
        ContinuationState {
            remaining: "",
            position: 1,
            line_number: 0,
            line_position: 1,
        },
    ));
    assert_eq!(result, expected);
}

#[test]
fn test_pany_fail() {
    let parser = pany(&['a', 'b', 'c']);
    let result = parser.parse("d".into());
    let expected = Err(Error::new(
        "a, b or c".to_string(),
        "d".to_string(),
        0,
        0,
        0,
    ));
    assert_eq!(result, expected);
}

#[test]
fn test_pmany_0() {
    let parser = pmany(pchar('a'));
    let result = parser.parse("b".into());
    let expected = Ok((
        Token {
            value: vec![],
            start: 0,
            length: 0,
        },
        ContinuationState {
            remaining: "b",
            position: 0,
            line_number: 0,
            line_position: 0,
        },
    ));
    assert_eq!(result, expected);
}

#[test]
fn test_pmany_1() {
    let parser = pmany(pchar('a'));
    let result = parser.parse("aaaa".into());
    let expected = Ok((
        Token {
            value: vec![
                Token::new('a', 0, 1),
                Token::new('a', 1, 1),
                Token::new('a', 2, 1),
                Token::new('a', 3, 1),
            ],
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
fn test_pmany_2() {
    let parser = pmany(pchar('a'));
    let result = parser.parse("aaab".into());
    let expected = Ok((
        Token {
            value: vec![
                Token::new('a', 0, 1),
                Token::new('a', 1, 1),
                Token::new('a', 2, 1),
            ],
            start: 0,
            length: 3,
        },
        ContinuationState {
            remaining: "b",
            position: 3,
            line_number: 0,
            line_position: 3,
        },
    ));
    assert_eq!(result, expected);
}

#[test]
fn test_between() {
    let parser = pbetween(pchar('('), pmany(pchar('a')), pchar(')'));
    let result = parser.parse("(aaa)".into());
    let expected = Ok((
        Token {
            value: vec![
                Token::new('a', 1, 1),
                Token::new('a', 2, 1),
                Token::new('a', 3, 1),
            ],
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
fn test_pmany1() {
    let parser = pmany1(pchar('1'));
    let result = parser.parse("0".into());
    let expected = Err(Error::new(
        "1 or more".to_string(),
        "0".to_string(),
        0,
        0,
        0,
    ));

    assert_eq!(result, expected);
}

#[test]
fn test_psepby() {
    let parser = psepby(pchar('1'), pchar(','));
    let result = parser.parse("1,1,1".into());
    let expected = Ok((
        Token {
            value: vec![
                Token::new('1', 0, 1),
                Token::new('1', 2, 1),
                Token::new('1', 4, 1),
            ],
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
fn test_psepby_missing_trail() {
    let parser = psepby(pchar('1'), pchar(','));
    let result = parser.parse("1,1,".into());
    let expected = Err(Error::new("1".to_string(), "".to_string(), 4, 0, 4));
    assert_eq!(result, expected);
}
