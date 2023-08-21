use super::*;

#[derive(Clone)]
struct AtLeastOneParser<'a, P, Output: Clone>
where
    P: Parser<'a, Vec<Token<Output>>>,
{
    parser: P,
    _phantom: std::marker::PhantomData<&'a Output>,
}

impl<'a, P, Output: Clone> Parser<'a, Vec<Token<Output>>> for AtLeastOneParser<'a, P, Output>
where
    P: Parser<'a, Vec<Token<Output>>>,
{
    fn parse(&self, input: ContinuationState<'a>) -> ParseResult<'a, Vec<Token<Output>>> {
        let result = self.parser.parse(input.clone());
        match result {
            Ok((token, cont)) => {
                if token.length == 0 {
                    Err(Error::new(
                        "1 or more".into(),
                        cont.remaining.to_string(),
                        input.position,
                        input.line_number,
                        input.line_position,
                    ))
                } else {
                    Ok((token, cont))
                }
            }
            Err(err) => Err(err),
        }
    }
}

pub(crate) fn p1<'a, Output: Clone + 'a>(
    parser: impl Parser<'a, Vec<Token<Output>>> + 'a,
) -> impl Parser<'a, Vec<Token<Output>>> {
    {
        AtLeastOneParser {
            parser,
            _phantom: std::marker::PhantomData,
        }
    }
}
