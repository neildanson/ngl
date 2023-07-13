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

pub fn pchar<'a>(
    c: char,
    state: ContinuationState<'a>,
) -> Result<(Token<char>, ContinuationState<'a>), String> {
    let mut chars = state.remaining.chars();
    match chars.next() {
        Some(letter) if letter == c => {
            let parser_state = state.advance(1);
            Ok((Token::new(c, state.position, 1), parser_state))
        }
        Some(letter) => Err(format_error(c, letter, &state)),
        None => Err(format_error(c, "EOF", &state)),
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_pchar_eof() {
        let result = pchar('H', "".into());
        let expected = Err("Expected 'H' but got 'EOF' at 0".to_string());
        assert_eq!(result, expected);
    }

    #[test]
    fn test_pchar_wrong_letter() {
        let result = pchar('H', "c".into());
        let expected = Err("Expected 'H' but got 'c' at 0".to_string());
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

    /*#[test]

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
