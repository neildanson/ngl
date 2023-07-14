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
    absolute_position: usize,
    line_number: usize,
    line_position: usize,
}

impl<'a> ContinuationState<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            remaining: input,
            absolute_position: 0,
            line_number: 0,
            line_position: 0,
        }
    }

    fn advance(&self, abs: usize, line: usize) -> Self {
        Self {
            remaining: &self.remaining[abs..],
            absolute_position: self.absolute_position + abs,
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
    pub position: usize, //TODO add lines and columns
}

impl Error {
    pub fn new(expected: String, actual: String, position: usize) -> Self {
        Self {
            expected,
            actual,
            position,
        }
    }
    fn format_error(&self) -> String {
        format!(
            "Expected '{}' but got '{}' at {}",
            self.expected, self.actual, self.position
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

pub fn pchar<'a>(c: char, state: ContinuationState<'a>) -> ParseResult<'a, char> {
    let mut chars = state.remaining.chars();
    match chars.next() {
        Some(letter) if letter == c => {
            let new_line = if letter == '\n' { 1 } else { 0 };
            let parser_state = state.advance(1, new_line);
            Ok((Token::new(c, state.absolute_position, 1), parser_state))
        }
        Some(letter) => Err(Error::new(
            c.to_string(),
            letter.to_string(),
            state.absolute_position,
        )),
        None => Err(Error::new(
            c.to_string(),
            "EOF".to_string(),
            state.absolute_position,
        )),
    }
}

#[macro_export]
macro_rules! pchar {
    ($value1:expr) => {{
        |cont| pchar($value1, cont)
    }};
}

#[macro_export]
macro_rules! pthen {
    ($parser1 : expr, $parser2 : expr) => {{
        move |input| {
            let result1 = $parser1(input);
            result1.and_then(|(token1, state1)| {
                let result2 = $parser2(state1);
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
    }};
}

#[macro_export]
macro_rules! por {
    ($parser1 : expr, $parser2 : expr) => {{
        move |input| {
            let result1 = $parser1(input);
            result1.or_else(|_error1| $parser2(input))
        }
    }};
}

#[macro_export]
macro_rules! poptional {
    ($parser1 : expr) => {{
        move |input| {
            let result1 = $parser1(input);
            match result1 {
                Ok((token, state)) => Ok((
                    Token::new(Some(token.value), token.start, token.length),
                    state,
                )),
                Err(_error1) => Ok((Token::new(None, 0, 0), input)),
            }
        }
    }};
}

#[macro_export]
macro_rules! pmap {
    ($parser1 : expr, $f : expr) => {{
        move |input| {
            let result1 = $parser1(input);
            result1.map(|(token, state)| {
                let result = $f(token.value);
                let token = Token::new(result, token.start, token.length);
                (token, state)
            })
        }
    }};
}

mod tests {
    use super::*;
    #[test]
    fn test_pchar_eof() {
        let parser = pchar!('H');
        let result = parser("".into());
        let expected = Err(Error::new("H".to_string(), "EOF".to_string(), 0));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_pchar_wrong_letter() {
        let parser = pchar!('H');
        let result = parser("c".into());
        let expected = Err(Error::new("H".to_string(), "c".to_string(), 0));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_pchar_success() {
        let parser = pchar!('H');
        let result = parser("H".into());
        let expected = Ok((
            Token {
                value: 'H',
                start: 0,
                length: 1,
            },
            ContinuationState {
                remaining: "",
                absolute_position: 1,
                line_number: 0,
                line_position: 1,
            },
        ));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_pthen_success_1() {
        let parser = pthen!(pchar!('H'), pchar!('e'));
        let result = parser("Hello".into());
        let expected = Ok((
            Token {
                value: ('H', 'e'),
                start: 0,
                length: 2,
            },
            ContinuationState {
                remaining: "llo",
                absolute_position: 2,
                line_number: 0,
                line_position: 2,
            },
        ));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_pthen_success_2() {
        let parser = pthen!(pchar!('H'), pchar!('e'));
        let result = parser("He".into());
        let expected = Ok((
            Token {
                value: ('H', 'e'),
                start: 0,
                length: 2,
            },
            ContinuationState {
                remaining: "",
                absolute_position: 2,
                line_number: 0,
                line_position: 2,
            },
        ));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_por_success_1() {
        let parser = por!(pchar!('H'), pchar!('h'));
        let result = parser("H".into());
        let expected = Ok((
            Token {
                value: 'H',
                start: 0,
                length: 1,
            },
            ContinuationState {
                remaining: "",
                absolute_position: 1,
                line_number: 0,
                line_position: 1,
            },
        ));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_por_success_2() {
        let parser = por!(pchar!('H'), pchar!('h'));
        let result = parser("h".into());
        let expected = Ok((
            Token {
                value: 'h',
                start: 0,
                length: 1,
            },
            ContinuationState {
                remaining: "",
                absolute_position: 1,
                line_number: 0,
                line_position: 1,
            },
        ));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_pmap_success() {
        let parser = pmap!(pchar!('T'), |_| true);
        let result = parser("T".into());
        let expected = Ok((
            Token {
                value: true,
                start: 0,
                length: 1,
            },
            ContinuationState {
                remaining: "",
                absolute_position: 1,
                line_number: 0,
                line_position: 1,
            },
        ));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_poptional_success() {
        let parser = poptional!(pchar!('T'));
        let result: ParseResult<Option<char>> = parser("T".into());
        let expected = Ok((
            Token {
                value: Some('T'),
                start: 0,
                length: 1,
            },
            ContinuationState {
                remaining: "",
                absolute_position: 1,
                line_number: 0,
                line_position: 1,
            },
        ));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_poptional_success_with_failure() {
        let parser = poptional!(pchar!('h'));
        let result = parser("T".into());
        let expected: ParseResult<Option<char>> = Ok((
            Token {
                value: None,
                start: 0,
                length: 0,
            },
            ContinuationState {
                remaining: "T",
                absolute_position: 0,
                line_number: 0,
                line_position: 0,
            },
        ));
        assert_eq!(result, expected);
    }

    /*
    #[test]

    fn test_pstring_eof() {
        let h_parser = pstring("Hello");
        let result = h_parser.parse("".into());
        let expected = Err("Expected 'Hello' but got 'EOF' at 0".to_string());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_pstring_wrong_letter() {
        let h_parser = pstring("Hello");
        let result = h_parser.parse("c".into());
        let expected = Err("Expected 'Hello' but got 'c' at 0".to_string());
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
            },
        ));
        assert_eq!(result, expected);
    }*/
}
