use super::*;

#[derive(Clone)]
struct ThenParser<
    'a,
    Left: Clone + 'a,
    Right: Clone + 'a,
    P1: Parser<'a, Left>,
    P2: Parser<'a, Right>,
> {
    parser1: P1,
    parser2: P2,
    _phantom: std::marker::PhantomData<&'a (Left, Right)>,
}

impl<'a, Left: Clone, Right: Clone, P1, P2> Parser<'a, (Token<Left>, Token<Right>)>
    for ThenParser<'a, Left, Right, P1, P2>
where
    P1: Parser<'a, Left>,
    P2: Parser<'a, Right>,
{
    fn parse(&self, input: ContinuationState<'a>) -> ParseResult<'a, (Token<Left>, Token<Right>)> {
        let result1 = self.parser1.parse(input);
        result1.and_then(|(token1, state1)| {
            let result2 = self.parser2.parse(state1);
            result2.map(|(token2, state2)| {
                let start = token1.start;

                let end_token1 = token1.start + token1.length;
                let gap = token2.start - end_token1;
                let length = gap + token1.length + token2.length;
                let token = Token::new((token1, token2), start, length);
                (token, state2)
            })
        })
    }
}

pub(crate) fn pthen<'a, Left: Clone + 'a, Right: Clone + 'a>(
    parser1: impl Parser<'a, Left> + 'a,
    parser2: impl Parser<'a, Right> + 'a,
) -> impl Parser<'a, (Token<Left>, Token<Right>)> {
    ThenParser {
        parser1,
        parser2,
        _phantom: std::marker::PhantomData,
    }
}
