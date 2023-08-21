use super::*;

#[derive(Clone)]
struct MapParser<'a, Input: Clone + 'a, Output: Clone + 'a, P: Parser<'a, Input>, F>
where
    F: Fn(Input) -> Output,
    F: Clone + 'a,
{
    parser: P,
    f: F,
    _phantom: std::marker::PhantomData<&'a (Input, Output)>,
}

impl<'a, Input: Clone + 'a, Output: Clone + 'a, P: Parser<'a, Input>, F> Parser<'a, Output>
    for MapParser<'a, Input, Output, P, F>
where
    F: Fn(Input) -> Output,
    F: Clone + 'a,
{
    fn parse(&self, input: ContinuationState<'a>) -> ParseResult<'a, Output> {
        let result = self.parser.parse(input);
        result.map(|(token, state)| {
            let result = (self.f)(token.value);
            let token = Token::new(result, token.start, token.length);
            (token, state)
        })
    }
}

pub(crate) fn pmap<'a, Input: Clone + 'a, Output: Clone + 'a, F>(
    parser: impl Parser<'a, Input> + 'a,
    f: F,
) -> impl Parser<'a, Output>
where
    F: Fn(Input) -> Output,
    F: Clone + 'a,
{
    MapParser {
        parser,
        f,
        _phantom: std::marker::PhantomData,
    }
}
