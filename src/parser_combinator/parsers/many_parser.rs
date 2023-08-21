use super::*;

#[derive(Clone)]
struct ManyParser<'a, Output: Clone + 'a, P: Parser<'a, Output>> {
    parser: P,
    _phantom: std::marker::PhantomData<&'a Output>,
}

impl<'a, Output: Clone + 'a, P> Parser<'a, Vec<Token<Output>>> for ManyParser<'a, Output, P>
where
    P: Parser<'a, Output>,
{
    fn parse(&self, input: ContinuationState<'a>) -> ParseResult<'a, Vec<Token<Output>>> {
        let mut results = Vec::new();
        let mut cont = input;
        let mut error = None;
        while error.is_none() {
            let result = self.parser.parse(cont);
            match result {
                Ok((token, state)) => {
                    results.push(token);
                    cont = state;
                }
                Err(err) => {
                    error = Some(err);
                }
            }
        }

        let len = results.len();
        match error {
            Some(_) => Ok((Token::new(results, input.position, len), cont)),
            None => Ok((Token::new(results, input.position, len), input)),
        }
    }
}

pub(crate) fn pmany<'a, Output: Clone + 'a>(
    parser: impl Parser<'a, Output> + 'a,
) -> impl Parser<'a, Vec<Token<Output>>> {
    {
        ManyParser {
            parser,
            _phantom: std::marker::PhantomData,
        }
    }
}
