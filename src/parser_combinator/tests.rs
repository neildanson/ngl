use super::*;
#[test]
fn test_pchar_eof() {
    let parser = pchar('H');
    let result = parser.parse("".into());
    let expected = Err(Error::new(Expected::Char('H'), "".to_string(), 0, 0, 0));
    assert_eq!(result, expected);
}

#[test]
fn test_pchar_wrong_letter() {
    let parser = pchar('H');
    let result = parser.parse("c".into());
    let expected = Err(Error::new(Expected::Char('H'), "c".to_string(), 0, 0, 0));
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
    let parser = pchar('H').then(pchar('e'));
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
    let parser = pchar('H').then(pchar('e'));
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
    let parser = pchar('H').or(pchar('h'));
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
    let parser = pchar('H').or(pchar('h'));
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
    let parser = pchar('H').or(pchar('h'));
    let result = parser.parse("e".into());
    let expected = Err(Error::new(
        Expected::Or(Box::new('H'.into()), Box::new('h'.into())),
        "e".to_string(),
        0,
        0,
        0,
    ));
    assert_eq!(result, expected);
}

#[test]
fn test_pmap_success() {
    let parser = pchar('T').map(|_| true);
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
    let parser = pchar('T').optional();
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
    let parser = pchar('h').optional();
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
    let expected = Err(Error::new("Hello".into(), "Hell".to_string(), 4, 0, 4));
    assert_eq!(result, expected);
}

#[test]
fn test_pstring_wrong_letter() {
    let h_parser = pstring("Hello");
    let result = h_parser.parse("c".into());
    let expected = Err(Error::new("Hello".into(), "c".to_string(), 0, 0, 0));
    assert_eq!(result, expected);
}

#[test]
fn test_pstring_wrong_letter_after_other_parse() {
    let parser1 = pchar('c').then(pchar('w'));
    let parser = parser1.then(pstring("Hello"));
    let result = parser.parse("cwrong".into());
    let expected = Err(Error::new("Hello".into(), "r".to_string(), 2, 0, 2));
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
    let parser1 = pchar('c').then(pstring("Hello"));
    let parser = parser1.then(pchar('w'));
    let result = parser.parse("cHelloX".into());
    let expected = Err(Error::new(Expected::Char('w'), "X".to_string(), 6, 0, 6));
    assert_eq!(result, expected);
}

#[test]
fn test_correct_line_number_on_error() {
    let parser = pchar('\n').then(pchar('\n'));
    let parser = parser.then(pchar('a'));
    let result = parser.parse("\n\nb".into());
    let expected = Err(Error::new(Expected::Char('a'), "b".to_string(), 2, 2, 0));
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
    let expected = Err(Error::new(
        Expected::Or(Box::new('a'.into()), Box::new('b'.into())),
        "c".to_string(),
        0,
        0,
        0,
    ));
    assert_eq!(result, expected);
}

#[test]
fn test_pchoice_fail_macro() {
    let parser = pchoice!(pchar('a'), pchar('b'), pchar('c'));
    let result = parser.parse("d".into());
    let expected = Err(Error::new(
        Expected::Or(
            Box::new('a'.into()),
            Box::new(Expected::Or(Box::new('b'.into()), Box::new('c'.into()))),
        ),
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
fn test_pws_success() {
    let parser = pws();
    let result = parser.parse(" ".into());
    let expected = Ok((
        Token {
            value: (),
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
fn test_pws_fail() {
    let parser = pws();
    let result = parser.parse("d".into());
    let expected = Err(Error::new(Expected::Char(' '), "d".to_string(), 0, 0, 0));
    assert_eq!(result, expected);
}

#[test]
fn test_pany_fail() {
    let parser = pany(&['a', 'b', 'c']);
    let result = parser.parse("d".into());
    let expected = Err(Error::new("a, b or c".into(), "d".to_string(), 0, 0, 0));
    assert_eq!(result, expected);
}

#[test]
fn test_pmany_0() {
    let parser = pchar('a').many();
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
    let parser = pchar('a').many();
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
    let parser = pchar('a').many();
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
    let parser = pchar('a').many().between(pchar('('), pchar(')'));
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
    let parser = pchar('1').many1();
    let result = parser.parse("0".into());
    let expected = Err(Error::new("1 or more".into(), "0".to_string(), 0, 0, 0));

    assert_eq!(result, expected);
}

#[test]
fn test_psepby() {
    let parser = pchar('1').sep_by(pchar(','));
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
    let parser = pchar('1').sep_by(pchar(','));
    let result = parser.parse("1,1,".into());
    let expected = Err(Error::new(Expected::Char('1'), "".to_string(), 4, 0, 4));
    assert_eq!(result, expected);
}
