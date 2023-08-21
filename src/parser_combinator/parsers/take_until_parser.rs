use super::*;

#[derive(Clone)]
struct TakeUntilParser<'a, P, Until: Clone>
where
    P: Parser<'a, Until>,
{
    until: P,
    _phantom: std::marker::PhantomData<&'a Until>,
}

impl<'a, P, Until: Clone> Parser<'a, &'a str> for TakeUntilParser<'a, P, Until>
where
    P: Parser<'a, Until>,
{
    fn parse(&self, input: ContinuationState<'a>) -> ParseResult<'a, &'a str> {
        ptakeuntil_impl(&self.until, None, input)
    }
}

pub(crate) fn ptake_until<'a, Until: Clone + 'a>(
    until: impl Parser<'a, Until>,
) -> impl Parser<'a, &'a str> {
    TakeUntilParser {
        until,
        _phantom: std::marker::PhantomData,
    }
}

//TODO deal with case where string is never termianted
fn ptakeuntil_impl<'a, Until: Clone + 'a>(
    until: &impl Parser<'a, Until>,
    start: Option<ContinuationState<'a>>,
    input: ContinuationState<'a>,
) -> ParseResult<'a, &'a str> {
    let result = until.parse(input.clone());
    let start = start.unwrap_or(input.clone());
    match result {
        Ok((_, cont)) => {
            let len = cont.position - start.position - 1;
            Ok((
                Token::new(&start.remaining[0..len], start.position, len),
                cont,
            ))
        }
        Err(_) => {
            let cont = input.advance(1, false); //TODO line advances
            ptakeuntil_impl(until, Some(start), cont)
        }
    }
}
