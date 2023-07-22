use std::fmt::{self, Debug, Display, Formatter};

#[derive(Debug, PartialEq)]
pub struct Token<T> {
    pub value: T,
    pub start: usize,
    pub length: usize,
}

impl<T> Token<T> {
    pub fn new(value: T, start: usize, length: usize) -> Self {
        Self {
            value,
            start,
            length,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ContinuationState<'a> {
    remaining: &'a str,
    position: usize,
    line_number: usize,
    line_position: usize,
}

impl<'a> ContinuationState<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            remaining: input,
            position: 0,
            line_number: 0,
            line_position: 0,
        }
    }

    fn advance(&self, abs: usize, line: usize) -> Self {
        Self {
            remaining: &self.remaining[abs..],
            position: self.position + abs,
            line_number: self.line_number + line,
            line_position: if line == 0 {
                self.line_position + abs
            } else {
                0
            },
        }
    }
}

impl<'a> From<&'a str> for ContinuationState<'a> {
    fn from(input: &'a str) -> Self {
        Self::new(input)
    }
}

#[derive(PartialEq)]
pub struct Error {
    pub expected: String,
    pub actual: String,
    pub position: usize,
    pub line_number: usize,
    pub line_position: usize,
}

impl Error {
    pub fn new(
        expected: String,
        actual: String,
        position: usize,
        line_number: usize,
        line_position: usize,
    ) -> Self {
        Self {
            expected,
            actual,
            position,
            line_number,
            line_position,
        }
    }
    fn format_error(&self) -> String {
        format!(
            "Expected '{}' but got '{}' at line: {}, column: {}",
            self.expected, self.actual, self.line_number, self.line_position
        )
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.format_error())
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.format_error())
    }
}

fn combine_error(error1: Error, error2: Error) -> Error {
    let expected = error1.expected.clone() + " or " + &error2.expected;
    let actual = error2.actual.clone();
    let position = error2.position;
    let line_number = error2.line_number;
    let line_position = error2.line_position;
    Error::new(expected, actual, position, line_number, line_position)
}

pub type ParseResult<'a, Output> = Result<(Token<Output>, ContinuationState<'a>), Error>;

pub fn pchar<'a>(c: char) -> impl Fn(ContinuationState<'a>) -> ParseResult<char> {
    move |input| {
        let mut chars = input.remaining.chars();
        match chars.next() {
            Some(letter) if letter == c => {
                let new_line = if letter == '\n' { 1 } else { 0 };
                let parser_state = input.advance(1, new_line);
                Ok((Token::new(c, input.position, 1), parser_state))
            }
            Some(letter) => Err(Error::new(
                c.to_string(),
                letter.to_string(),
                input.position,
                input.line_number,
                input.line_position,
            )),
            None => Err(Error::new(
                c.to_string(),
                "".to_string(),
                input.position,
                input.line_number,
                input.line_position,
            )),
        }
    }
}

pub fn pstring<'a>(value: &'a str) -> impl Fn(ContinuationState<'a>) -> ParseResult<'a, &'a str> {
    move |input| {
        let mut cont = input;
        let mut error = None;
        let mut success = Vec::new();
        for t in value.chars() {
            let parser = pchar(t);
            let result = parser(cont);
            match result {
                Ok((_, new_cont)) => {
                    success.push(t);
                    cont = new_cont
                }
                Err(err) => {
                    let actual = success.iter().collect::<String>() + &err.actual;
                    error = Some(Err(Error::new(
                        value.to_string(),
                        actual.to_string(),
                        err.position,
                        err.line_number,
                        err.line_position,
                    )));
                    break;
                }
            }
        }
        match error {
            Some(err) => err,
            None => Ok((Token::new(value, input.position, value.len()), cont)), //This seems to work, but I dont know why!
        }
    }
}

pub fn pthen<'a, T, U>(
    parser1: impl Fn(ContinuationState<'a>) -> ParseResult<T>,
    parser2: impl Fn(ContinuationState<'a>) -> ParseResult<U>,
) -> impl Fn(ContinuationState<'a>) -> ParseResult<(Token<T>, Token<U>)> {
    move |input| {
        let result1 = parser1(input);
        result1.and_then(|(token1, state1)| {
            let result2 = parser2(state1);
            result2.map(|(token2, state2)| {
                let start = token1.start;
                let length = token1.length + token2.length;
                let token = Token::new((token1, token2), start, length);
                (token, state2)
            })
        })
    }
}

pub fn por<'a, T>(
    parser1: impl Fn(ContinuationState<'a>) -> ParseResult<T>,
    parser2: impl Fn(ContinuationState<'a>) -> ParseResult<T>,
) -> impl Fn(ContinuationState<'a>) -> ParseResult<T> {
    move |input| {
        let result1 = parser1(input);
        result1.or_else(|error| {
            let result = parser2(input);
            match result {
                Ok((token, state)) => Ok((token, state)),
                Err(error2) => {
                    let error = Error::new(
                        error.expected + " or " + &error2.expected,
                        error2.actual,
                        error2.position,
                        error2.line_number,
                        error2.line_position,
                    );
                    Err(error)
                }
            }
        })
    }
}

pub fn poptional<'a, T>(
    parser: impl Fn(ContinuationState<'a>) -> ParseResult<T>,
) -> impl Fn(ContinuationState<'a>) -> ParseResult<Option<T>> {
    move |input| {
        let result1 = parser(input);
        match result1 {
            Ok((token, state)) => Ok((
                Token::new(Some(token.value), token.start, token.length),
                state,
            )),
            Err(_error1) => Ok((Token::new(None, input.position, 0), input)),
        }
    }
}

pub fn pmap<'a, T, U>(
    parser: impl Fn(ContinuationState<'a>) -> ParseResult<T>,
    f: impl Fn(T) -> U,
) -> impl Fn(ContinuationState<'a>) -> ParseResult<U> {
    move |input| {
        let result = parser(input);
        result.map(|(token, state)| {
            let result = f(token.value);
            let token = Token::new(result, token.start, token.length);
            (token, state)
        })
    }
}

#[macro_export]
macro_rules! pchoice {
    ($head:expr) => ({
        move |input| $head(input) //TODO - we should accumulate the errors for choice (ie "a" or "b" )
    });
    ($head:expr, $($tail:expr),*) => ({
        move |input| {
            let result1 = $head(input);
            result1.or_else(|error1|{
                let result = pchoice!($($tail),*)(input);
                result.map_err(|error2| combine_error(error1, error2))
            })
        }});
}

pub fn pany<'a>(
    valid_chars: &'a [char],
) -> impl Fn(ContinuationState<'a>) -> ParseResult<'a, char> {
    move |input| {
        for c in valid_chars {
            let result = pchar(*c)(input);
            match result {
                Ok((token, state)) => return Ok((token, state)),
                Err(_) => continue,
            }
        }

        let valid_chars_length = valid_chars.len();
        let error = if valid_chars_length >= 2 {
            let first = valid_chars
                .iter()
                .take(valid_chars.len() - 1)
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            first + " or " + &valid_chars.last().unwrap().to_string()
        } else if valid_chars_length == 1 {
            valid_chars.iter().next().unwrap().to_string()
        } else {
            "".to_string() //TODO - this should never happen
        };

        let actual = input.remaining.chars().next().unwrap_or(' ').to_string();

        Err(Error::new(
            error,
            actual,
            input.position,
            input.line_number,
            input.line_position,
        ))
    }
}

pub fn pmany<'a, T>(
    parser: impl Fn(ContinuationState<'a>) -> ParseResult<T>,
) -> impl Fn(ContinuationState<'a>) -> ParseResult<'a, Vec<Token<T>>> {
    move |input| {
        let mut results = Vec::new();
        let mut cont = input;
        let mut error = None;
        loop {
            let result = parser(cont);
            match result {
                Ok((token, state)) => {
                    results.push(token);
                    cont = state;
                }
                Err(err) => {
                    error = Some(err);
                    break;
                }
            }
        }

        let len = results.len();
        match error {
            Some(_) => Ok((Token::new(results, input.position, len), cont)),
            None => Ok((Token::new(results, 0, len), input)),
        }
    }
}

pub fn psepby<'a, T, U>(
    parser: impl Fn(ContinuationState<'a>) -> ParseResult<'a, T>,
    separator: impl Fn(ContinuationState<'a>) -> ParseResult<'a, U>,
) -> impl Fn(ContinuationState<'a>) -> ParseResult<'a, Vec<Token<T>>> {
    let parser_combined = pleft(pthen(parser, separator));
    let parser_many = pmany(parser_combined);
    move |input| {
        let result = { parser_many(input) };
        match result {
            Ok((mut token, cont)) => 
                {
                   let result = parser(cont); 
                   match result {
                    Ok((token_last, cont)) => {
                        token.value.push(token_last);
                        Ok((token, cont))
                    }
                    ,
                    Err(err) => Err(err)
                   }
                },
            Err(err) => Err(err)
        }
    }
}

pub fn pleft<'a, T, U>(
    parser: impl Fn(ContinuationState<'a>) -> ParseResult<(Token<T>, Token<U>)>,
) -> impl Fn(ContinuationState<'a>) -> ParseResult<T> {
    move |input| {
        let result = parser(input);
        result.map(|(token, cont)| {
            let token = token.value.0;
            (token, cont)
        })
    }
}

pub fn pright<'a, T, U>(
    parser: impl Fn(ContinuationState<'a>) -> ParseResult<(Token<T>, Token<U>)>,
) -> impl Fn(ContinuationState<'a>) -> ParseResult<U> {
    move |input| {
        let result = parser(input);
        result.map(|(token, cont)| {
            let token = token.value.1;
            (token, cont)
        })
    }
}

pub fn pbetween<'a, T, U, V>(
    parser1: impl Fn(ContinuationState<'a>) -> ParseResult<T>,
    parser2: impl Fn(ContinuationState<'a>) -> ParseResult<U>,
    parser3: impl Fn(ContinuationState<'a>) -> ParseResult<V>,
) -> impl Fn(ContinuationState<'a>) -> ParseResult<U> {
    let parser = pthen(parser1, pthen(parser2, parser3));
    let parser = pright(parser); //Skip T
    pleft(parser) //Ignore U
}

pub fn p1<'a, T>(
    parser: impl Fn(ContinuationState<'a>) -> ParseResult<'a, Vec<Token<T>>>,
) -> impl Fn(ContinuationState<'a>) -> ParseResult<'a, Vec<Token<T>>> {
    move |input| {
        let result = parser(input);
        match result {
            Ok((token, cont)) => {
                if token.length == 0 {
                    Err(Error::new(
                        "1 or more".to_string(),
                        cont.remaining.to_string(),
                        input.position,
                        input.line_number,
                        input.line_position,
                    ))
                } else {
                    Ok((token, cont))
                }
            }
            Err(err) => Err(err),
        }
    }
}

pub fn pmany1<'a, T>(
    parser: impl Fn(ContinuationState<'a>) -> ParseResult<T>,
) -> impl Fn(ContinuationState<'a>) -> ParseResult<'a, Vec<Token<T>>> {
    p1(pmany(parser))
}

mod tests {
    use super::*;
    #[test]
    fn test_pchar_eof() {
        let parser = pchar('H');
        let result = parser("".into());
        let expected = Err(Error::new("H".to_string(), "".to_string(), 0, 0, 0));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_pchar_wrong_letter() {
        let parser = pchar('H');
        let result = parser("c".into());
        let expected = Err(Error::new("H".to_string(), "c".to_string(), 0, 0, 0));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_pchar_success() {
        let parser = pchar('H');
        let result = parser("H".into());
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
        let result = parser("Hello".into());
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
        let result = parser("He".into());
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
        let result = parser("H".into());
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
        let result = parser("h".into());
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
        let result = parser("e".into());
        let expected = Err(Error::new("H or h".to_string(), "e".to_string(), 0, 0, 0));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_pmap_success() {
        let parser = pmap(pchar('T'), |_| true);
        let result = parser("T".into());
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
        let result: ParseResult<Option<char>> = parser("T".into());
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
        let result = parser("T".into());
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
        let result = h_parser("Hell".into());
        let expected = Err(Error::new("Hello".to_string(), "Hell".to_string(), 4, 0, 4));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_pstring_wrong_letter() {
        let h_parser = pstring("Hello");
        let result = h_parser("c".into());
        let expected = Err(Error::new("Hello".to_string(), "c".to_string(), 0, 0, 0));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_pstring_wrong_letter_after_other_parse() {
        let parser1 = pthen(pchar('c'), pchar('w'));
        let parser = pthen(parser1, pstring("Hello"));
        let result = parser("cwrong".into());
        let expected = Err(Error::new("Hello".to_string(), "r".to_string(), 2, 0, 2));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_pstring_success() {
        let h_parser = pstring("Hello");
        let result = h_parser("Hello".into());
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
        let result = parser("cHelloX".into());
        let expected = Err(Error::new("w".to_string(), "X".to_string(), 6, 0, 6));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_correct_line_number_on_error() {
        let parser = pthen(pchar('\n'), pchar('\n'));
        let parser = pthen(parser, pchar('a'));
        let result = parser("\n\nb".into());
        let expected = Err(Error::new("a".to_string(), "b".to_string(), 2, 2, 0));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_pchoice_success() {
        let parser = pchoice!(pchar('a'), pchar('b'));
        let result = parser("a".into());
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
        let parser = pchoice!(pchar('a'), pchar('b'));
        let result = parser("c".into());
        let expected = Err(Error::new("a or b".to_string(), "c".to_string(), 0, 0, 0));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_pany_success() {
        let parser = pany(&['a', 'b', 'c']);
        let result = parser("b".into());
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
        let result = parser("d".into());
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
        let result = parser("b".into());
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
        let result = parser("aaaa".into());
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
        let result = parser("aaab".into());
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
        let result = parser("(aaa)".into());
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
        let result = parser("0".into());
        let expected = Err(Error::new(
            "1 or more".to_string(),
            "0".to_string(),
            0,
            0,
            0,
        ));

        assert_eq!(result, expected);
    }
}
