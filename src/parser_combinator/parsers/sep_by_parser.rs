use super::*;

#[derive(Clone)]
struct SepByParser<'a, P, S, Output: Clone + 'a, Seperator: Clone + 'a>
where
    P: Parser<'a, Output>,
    S: Parser<'a, Seperator>,
{
    parser: P,
    separator: S,
    _phantom: std::marker::PhantomData<&'a (Output, Seperator)>,
}

impl<'a, P, S, Output: Clone + 'a, Seperator: Clone + 'a> Parser<'a, Vec<Token<Output>>>
    for SepByParser<'a, P, S, Output, Seperator>
where
    P: Parser<'a, Output> + 'a,
    S: Parser<'a, Seperator> + 'a,
{
    fn parse(&self, input: ContinuationState<'a>) -> ParseResult<'a, Vec<Token<Output>>> {
        let parser = self
            .parser
            .clone()
            .then(self.separator.clone())
            .left()
            .many()
            .then(self.parser.clone());

        let parser = parser.map(|(mut tokens, token)| {
            tokens.value.push(token);
            tokens.value
        });
        parser.parse(input)
    }
}

pub(crate) fn psepby<'a, Output: Clone + 'a, Seperator: Clone + 'a>(
    parser: impl Parser<'a, Output> + 'a,
    separator: impl Parser<'a, Seperator> + 'a,
) -> impl Parser<'a, Vec<Token<Output>>> {
    SepByParser {
        parser,
        separator,
        _phantom: std::marker::PhantomData,
    }
}
