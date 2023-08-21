use super::*;
#[derive(Clone)]
struct ClosureParser<'a, Output, F>
where
    F: Fn(ContinuationState<'a>) -> ParseResult<'a, Output>,
{
    parser: F,
    _phantom: std::marker::PhantomData<&'a Output>,
}

pub fn parser_from_fn<'a, Output: Clone + 'a, F: Clone>(parser: F) -> impl Parser<'a, Output>
where
    F: Fn(ContinuationState<'a>) -> ParseResult<'a, Output>,
{
    ClosureParser {
        parser,
        _phantom: std::marker::PhantomData,
    }
}

impl<'a, Output: Clone, F> Parser<'a, Output> for ClosureParser<'a, Output, F>
where
    F: Fn(ContinuationState<'a>) -> ParseResult<'a, Output> + Clone,
{
    fn parse(&self, input: ContinuationState<'a>) -> ParseResult<'a, Output> {
        (self.parser)(input)
    }
}
