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

pub type ParseResult<'a, Output> = Result<(Token<Output>, ContinuationState<'a>), Error>;

pub fn pchar<'a>(c: char) -> impl Fn(ContinuationState<'a>) -> ParseResult<char> {
    move |state| {
        let mut chars = state.remaining.chars();
        match chars.next() {
            Some(letter) if letter == c => {
                let new_line = if letter == '\n' { 1 } else { 0 };
                let parser_state = state.advance(1, new_line);
                Ok((Token::new(c, state.position, 1), parser_state))
            }
            Some(letter) => Err(Error::new(
                c.to_string(),
                letter.to_string(),
                state.position,
                state.line_number,
                state.line_position,
            )),
            None => Err(Error::new(
                c.to_string(),
                "".to_string(),
                state.position,
                state.line_number,
                state.line_position,
            )),
        }
    }
}

pub fn pstring<'a>(value: &'a str) -> impl Fn(ContinuationState<'a>) -> ParseResult<'a, &'a str> {
    |input| {
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
            None => Ok((Token::new(value, 0, value.len()), cont)), //This seems to work, but I dont know why!
        }
    }
}

pub fn pthen<'a, T, U>(
    parser1: impl Fn(ContinuationState<'a>) -> ParseResult<T>,
    parser2: impl Fn(ContinuationState<'a>) -> ParseResult<U>,
) -> impl Fn(ContinuationState<'a>) -> ParseResult<(T, U)> {
    move |input| {
        let result1 = parser1(input);
        result1.and_then(|(token1, state1)| {
            let result2 = parser2(state1);
            result2.map(|(token2, state2)| {
                let token = Token::new(
                    (token1.value, token2.value),
                    token1.start,
                    token1.length + token2.length,
                );
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
        result1.or_else(|_error1| parser2(input))
    }
}

pub fn poptional<'a, T>(
    parser1: impl Fn(ContinuationState<'a>) -> ParseResult<T>,
) -> impl Fn(ContinuationState<'a>) -> ParseResult<Option<T>> {
    move |input| {
        let result1 = parser1(input);
        match result1 {
            Ok((token, state)) => Ok((
                Token::new(Some(token.value), token.start, token.length),
                state,
            )),
            Err(_error1) => Ok((Token::new(None, 0, 0), input)),
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
            result1.or_else(|_error1| pchoice!($($tail),*)(input))
        }});
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
                value: ('H', 'e'),
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
                value: ('H', 'e'),
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
    fn test_pchoice_fail() {
        let parser = pchoice!(pchar('a'), pchar('b'));
        let result = parser("c".into());
        let expected = Err(Error::new("b".to_string(), "c".to_string(), 0, 0, 0));
        assert_eq!(result, expected);
    }
}
