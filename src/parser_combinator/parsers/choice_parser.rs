use super::*;

#[derive(Clone)]
struct ChoiceParser<'a, P, Output: Clone + 'a>
where
    P: Parser<'a, Output>,
{
    parsers: Vec<P>,
    _phantom: std::marker::PhantomData<&'a Output>,
}

impl<'a, P, Output: Clone> Parser<'a, Output> for ChoiceParser<'a, P, Output>
where
    P: Parser<'a, Output>,
{
    fn parse(&self, input: ContinuationState<'a>) -> ParseResult<'a, Output> {
        let mut errors = Vec::new();
        for parser in self.parsers.iter() {
            let result = parser.parse(input.clone());
            match result {
                Ok((token, cont)) => return Ok((token, cont)),
                Err(err) => errors.push(err),
            }
        }

        let mut error = errors.remove(0);
        for err in errors.into_iter() {
            error = error + err;
        }

        Err(error)
    }
}

pub fn pchoice<'a, Output: Clone + 'a>(
    parsers: Vec<impl Parser<'a, Output>>,
) -> impl Parser<'a, Output> {
    ChoiceParser {
        parsers,
        _phantom: std::marker::PhantomData,
    }
}
