use super::*;

#[derive(Clone)]
struct WhitespaceParser;

impl<'a> Parser<'a, ()> for WhitespaceParser {
    fn parse(&self, input: ContinuationState<'a>) -> ParseResult<'a, ()> {
        let next_char = input.remaining.chars().next();
        if let Some(next_char) = next_char {
            if next_char.is_whitespace() {
                let parser_state = input.advance(1, next_char == '\n');
                return Ok((Token::new((), input.position, 1), parser_state));
            }
        }

        let actual = next_char.unwrap_or(' ').to_string();
        Err(Error::new(
            Expected::Char(' '),
            actual,
            input.position,
            input.line_number,
            input.line_position,
        ))
    }
}

pub fn pws<'a>() -> impl Parser<'a, ()> {
    WhitespaceParser
}
