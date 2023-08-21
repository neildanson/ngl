use super::*;

#[derive(Clone)]
struct OptionalParser<'a, Output: Clone + 'a, P: Parser<'a, Output>> {
    parser: P,
    _phantom: std::marker::PhantomData<&'a Output>,
}

impl<'a, Output: Clone, P> Parser<'a, Option<Output>> for OptionalParser<'a, Output, P>
where
    P: Parser<'a, Output>,
{
    fn parse(&self, input: ContinuationState<'a>) -> ParseResult<'a, Option<Output>> {
        let result1 = self.parser.parse(input.clone());
        match result1 {
            Ok((token, state)) => Ok((
                Token::new(Some(token.value), token.start, token.length),
                state,
            )),
            Err(_error1) => Ok((Token::new(None, input.position, 0), input)),
        }
    }
}

pub(crate) fn poptional<'a, Output: Clone + 'a>(
    parser: impl Parser<'a, Output> + 'a,
) -> impl Parser<'a, Option<Output>> {
    OptionalParser {
        parser,
        _phantom: std::marker::PhantomData,
    }
}
