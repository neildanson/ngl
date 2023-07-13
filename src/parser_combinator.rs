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
    position: usize, //TODO add line numbers
}

impl<'a> ContinuationState<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            remaining: input,
            position: 0,
        }
    }

    fn advance(&self, n: usize) -> Self {
        Self {
            remaining: &self.remaining[n..],
            position: self.position + n,
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
    expected: String,
    actual: String,
    position: usize,
}

impl Error {
    fn new(expected: String, actual: String, position: usize) -> Self {
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
            let parser_state = state.advance(1);
            Ok((Token::new(c, state.position, 1), parser_state))
        }
        Some(letter) => Err(Error::new(
            c.to_string(),
            letter.to_string(),
            state.position,
        )),
        None => Err(Error::new(c.to_string(), "EOF".to_string(), state.position)),
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
            match result1 {
                Ok((token1, state1)) => {
                    let result2 = $parser2(state1);
                    match result2 {
                        Ok((token2, state2)) => {
                            let token = Token::new(
                                (token1.value, token2.value),
                                token1.start,
                                token1.length + token2.length,
                            );
                            Ok((token, state2))
                        }
                        Err(e) => Err(e),
                    }
                }
                Err(e) => Err(e),
            }
        }
    }};
}

#[macro_export]
macro_rules! por {
    ($parser1 : expr, $parser2 : expr) => {{
        move |input| {
            let result1 = $parser1(input);
            match result1 {
                Err(error1) => {
                    let result2 = $parser2(input);
                    match result2 {
                        Ok((token2, state2)) => {
                            let token = Token::new(token2.value, token2.start, token2.length);
                            Ok((token, state2))
                        }
                        Err(_error2) => Err(error1), //TODO combine errors
                    }
                }
                Ok((token1, state1)) => Ok((token1, state1)),
            }
        }
    }};
}

#[macro_export]
macro_rules! pmap {
    ($parser1 : expr, $f : expr) => {{
        move |input| {
            let result1 = $parser1(input);
            match result1 {
                Ok((token, state)) => {
                    let result = $f(token.value);
                    let token = Token::new(result, token.start, token.length);
                    Ok((token, state))
                }
                Err(e) => Err(e),
            }
        }
    }};
}

mod tests {
    use super::*;

    #[test]
    fn test_pchar_eof() {
        let result = pchar('H', "".into());
        let expected = Err(Error::new("H".to_string(), "EOF".to_string(), 0));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_pchar_wrong_letter() {
        let result = pchar('H', "c".into());
        let expected = Err(Error::new("H".to_string(), "c".to_string(), 0));
        assert_eq!(result, expected);
    }

    #[test]
    fn test_pchar_success() {
        let result = pchar('H', "H".into());
        let expected = Ok((
            Token {
                value: 'H',
                start: 0,
                length: 1,
            },
            ContinuationState {
                remaining: "",
                position: 1,
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
                position: 2,
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
                position: 2,
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
                position: 1,
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
                position: 1,
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
                position: 1,
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
