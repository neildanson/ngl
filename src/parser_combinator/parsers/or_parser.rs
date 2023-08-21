use super::*;

#[derive(Clone)]
struct OrParser<'a, Output: Clone + 'a, P1: Parser<'a, Output>, P2: Parser<'a, Output>> {
    parser1: P1,
    parser2: P2,
    _phantom: std::marker::PhantomData<&'a Output>,
}

impl<'a, Output: Clone, P1, P2> Parser<'a, Output> for OrParser<'a, Output, P1, P2>
where
    P1: Parser<'a, Output>,
    P2: Parser<'a, Output>,
{
    fn parse(&self, input: ContinuationState<'a>) -> ParseResult<'a, Output> {
        let result1 = self.parser1.parse(input.clone());
        result1.or_else(|error| {
            let result = self.parser2.parse(input);
            match result {
                Ok((token, state)) => Ok((token, state)),
                Err(error2) => {
                    let error = Error::new(
                        error.expected + error2.expected,
                        error2.actual,
                        error2.position,
                        error2.line_number,
                        error2.line_position,
                    );
                    Err(error)
                }
            }
        })
    }
}

pub(crate) fn por<'a, Output: Clone + 'a>(
    parser1: impl Parser<'a, Output> + 'a,
    parser2: impl Parser<'a, Output> + 'a,
) -> impl Parser<'a, Output> {
    OrParser {
        parser1,
        parser2,
        _phantom: std::marker::PhantomData,
    }
}
