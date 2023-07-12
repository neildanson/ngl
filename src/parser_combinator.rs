use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub struct Token<T> {
    value: T,
    start: usize,
    length: usize,
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

#[derive(Debug, PartialEq)]
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

pub trait Parser<'a> {
    type Output;
    fn parse(
        &self,
        input: ContinuationState<'a>,
    ) -> Result<(Token<Self::Output>, ContinuationState<'a>), String>;
}

struct ParserFn<'a, Output> {
    parser:
        Rc<dyn Fn(ContinuationState<'a>) -> Result<(Token<Output>, ContinuationState<'a>), String>>,
}

impl<'a, Output> ParserFn<'a, Output> {
    pub fn new(
        parser: Rc<
            dyn Fn(ContinuationState<'a>) -> Result<(Token<Output>, ContinuationState<'a>), String>,
        >,
    ) -> Self {
        Self { parser }
    }
}

impl<'a, Output> Parser<'a> for ParserFn<'a, Output> {
    type Output = Output;
    fn parse(
        &self,
        input: ContinuationState<'a>,
    ) -> Result<(Token<Output>, ContinuationState<'a>), String> {
        (self.parser)(input)
    }
}

fn format_error<T, U>(expected: T, actual: U, state: &ContinuationState) -> String
where
    T: std::fmt::Display,
    U: std::fmt::Display,
{
    format!(
        "Expected '{}' but got '{}' at {}",
        expected, actual, state.position
    )
}

pub fn pchar<'a>(c: char) -> impl Parser<'a, Output = char> {
    ParserFn::new(Rc::new(move |state: ContinuationState| {
        let mut chars = state.remaining.chars();
        match chars.next() {
            Some(letter) if letter == c => {
                let parser_state = state.advance(1);
                Ok((Token::new(c, state.position, 1), parser_state))
            }
            Some(letter) => Err(format_error(c, letter, &state)),
            None => Err(format_error(c, "EOF", &state)),
        }
    }))
}

pub fn pstring<'a>(s: &'static str) -> impl Parser<'a, Output = &'a str> + 'a {
    ParserFn::new(Rc::new(move |state: ContinuationState| {
        let startswith = state.remaining.starts_with(s);
        if startswith {
            let parser_state = state.advance(s.len());
            Ok((Token::new(s, state.position, s.len()), parser_state))
        } else {
            let mut chars = state.remaining.chars();
            match chars.next() {
                Some(letter) => Err(format_error(s, letter, &state)),
                None => Err(format_error(s, "EOF", &state)),
            }
        }
    }))
}

mod tests {
    use super::*;

    #[test]
    fn test_pchar_eof() {
        let h_parser = pchar('H');
        let result = h_parser.parse("".into());
        let expected = Err("Expected 'H' but got 'EOF' at 0".to_string());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_pchar_wrong_letter() {
        let h_parser = pchar('H');
        let result = h_parser.parse("c".into());
        let expected = Err("Expected 'H' but got 'c' at 0".to_string());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_pchar_success() {
        let h_parser = pchar('H');
        let result = h_parser.parse("H".into());
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
    }
}
