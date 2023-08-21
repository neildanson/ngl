use super::*;

pub(crate) fn pchar_impl(c: char, input: ContinuationState<'_>) -> ParseResult<'_, char> {
    let mut chars = input.remaining.chars();
    match chars.next() {
        Some(letter) if letter == c => {
            let parser_state = input.advance(1, letter == '\n');
            Ok((Token::new(c, input.position, 1), parser_state))
        }
        Some(letter) => Err(Error::new(
            c.into(),
            letter.to_string(),
            input.position,
            input.line_number,
            input.line_position,
        )),
        None => Err(Error::new(
            c.into(),
            "".to_string(),
            input.position,
            input.line_number,
            input.line_position,
        )),
    }
}

#[derive(Clone)]
struct CharParser {
    value: char,
}

impl<'a> Parser<'a, char> for CharParser {
    fn parse(&self, input: ContinuationState<'a>) -> ParseResult<'a, char> {
        pchar_impl(self.value, input)
    }
}

/// Matches a single character.
/// ```
/// use ngl::parser_combinator::pchar;
/// use ngl::parser_combinator::Parser;
///
/// let a = pchar('a');
/// let result = a.parse("abc".into()).unwrap();
/// assert_eq!(result.0.value, 'a');
///
/// ```
pub fn pchar<'a>(value: char) -> impl Parser<'a, char> {
    CharParser { value }
}
