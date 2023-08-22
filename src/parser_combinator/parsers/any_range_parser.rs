use super::*;
use std::ops::RangeInclusive;

#[derive(Clone)]
struct AnyRangeParser {
    valid_chars: RangeInclusive<char>,
}

impl<'a> Parser<'a, char> for AnyRangeParser {
    fn parse(&self, input: ContinuationState<'a>) -> ParseResult<'a, char> {
        let next_char = input.remaining.chars().next();
        if let Some(next_char) = next_char {
            if self.valid_chars.contains(&next_char) {
                let parser_state = input.advance(1, next_char == '\n');
                return Ok((Token::new(next_char, input.position, 1), parser_state));
            }
        }

        let actual = if !input.remaining.is_empty() {
            &input.remaining[0..1]
        } else {
            " "
        };
        Err(Error::new(
            self.valid_chars.clone().into(),
            actual,
            input.position,
            input.line_number,
            input.line_position,
        ))
    }
}

pub fn pany_range<'a>(valid_chars: RangeInclusive<char>) -> impl Parser<'a, char> {
    AnyRangeParser { valid_chars }
}
