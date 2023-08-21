use super::*;
#[derive(Clone)]
struct AnyParser<'a> {
    valid_chars: &'a [char],
}

impl<'a> Parser<'a, char> for AnyParser<'a> {
    fn parse(&self, input: ContinuationState<'a>) -> ParseResult<'a, char> {
        for c in self.valid_chars.iter() {
            let result = pchar_impl(*c, input.clone());
            match result {
                Ok((token, state)) => return Ok((token, state)),
                Err(_) => continue,
            }
        }

        let actual = input.remaining.chars().next().unwrap_or(' ').to_string();

        Err(Error::new(
            self.valid_chars.into(),
            actual,
            input.position,
            input.line_number,
            input.line_position,
        ))
    }
}

pub fn pany(valid_chars: &[char]) -> impl Parser<char> {
    AnyParser { valid_chars }
}
