use super::*;

#[derive(Clone)]
struct StringParser<'a> {
    value: &'a str,
}

impl<'a> Parser<'a, &'a str> for StringParser<'a> {
    fn parse(&self, input: ContinuationState<'a>) -> ParseResult<'a, &'a str> {
        let mut cont = input;
        let mut error = None;
        for t in self.value.chars() {
            let result = char_parser::pchar_impl(t, cont);
            match result {
                Ok((_, new_cont)) => cont = new_cont,
                Err(err) => {
                    let length = err.position - input.position + 1;
                    let actual = if input.remaining.len() < length {
                        input.remaining[0..].to_string()
                    } else {
                        input.remaining[0..length].to_string()
                    };

                    error = Some(Err(Error::new(
                        self.value.into(),
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
            None => Ok((
                Token::new(self.value, input.position, self.value.len()),
                cont,
            )),
        }
    }
}

pub fn pstring(value: &str) -> impl Parser<'_, &str> {
    StringParser { value }
}
