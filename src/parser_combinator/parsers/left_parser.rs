use super::*;

#[derive(Clone)]
struct LeftParser<
    'a,
    Left: Clone + 'a,
    Right: Clone + 'a,
    P: Parser<'a, (Token<Left>, Token<Right>)>,
> where
    P: Parser<'a, (Token<Left>, Token<Right>)>,
{
    parser: P,
    _phantom: std::marker::PhantomData<&'a (Left, Right)>,
}

impl<'a, Left: Clone + 'a, Right: Clone + 'a, P: Parser<'a, (Token<Left>, Token<Right>)>>
    Parser<'a, Left> for LeftParser<'a, Left, Right, P>
{
    fn parse(&self, input: ContinuationState<'a>) -> ParseResult<'a, Left> {
        let result = self.parser.parse(input);
        result.map(|(token, cont)| {
            let token = token.value.0;
            (token, cont)
        })
    }
}

pub(crate) fn pleft<'a, Left: Clone + 'a, Right: Clone + 'a>(
    parser: impl Parser<'a, (Token<Left>, Token<Right>)> + 'a,
) -> impl Parser<'a, Left> {
    LeftParser {
        parser,
        _phantom: std::marker::PhantomData,
    }
}
