use super::*;

#[derive(Clone)]
struct RightParser<
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
    Parser<'a, Right> for RightParser<'a, Left, Right, P>
{
    fn parse(&self, input: ContinuationState<'a>) -> ParseResult<'a, Right> {
        let result = self.parser.parse(input);
        result.map(|(token, cont)| {
            let token = token.value.1;
            (token, cont)
        })
    }
}

pub(crate) fn pright<'a, Left: Clone + 'a, Right: Clone + 'a>(
    parser: impl Parser<'a, (Token<Left>, Token<Right>)> + 'a,
) -> impl Parser<'a, Right> {
    RightParser {
        parser,
        _phantom: std::marker::PhantomData,
    }
}
