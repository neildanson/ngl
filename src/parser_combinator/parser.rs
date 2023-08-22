use crate::{
    parser_combinator::continuation::ContinuationState, parser_combinator::error::*,
    parser_combinator::parsers::*, parser_combinator::token::Token,
};

pub type ParseResult<'a, Output> = Result<(Token<Output>, ContinuationState<'a>), Error<'a>>;

pub trait Parser<'a, Output: Clone + 'a>: Clone {
    fn parse(&self, input: ContinuationState<'a>) -> ParseResult<'a, Output>;
    fn then<NextOutput: Clone + 'a>(
        self,
        next: impl Parser<'a, NextOutput> + 'a,
    ) -> impl Parser<'a, (Token<Output>, Token<NextOutput>)>
    where
        Self: Sized + 'a,
    {
        pthen(self, next)
    }

    fn or(self, next: impl Parser<'a, Output> + 'a) -> impl Parser<'a, Output>
    where
        Self: Sized + 'a,
    {
        por(self, next)
    }

    fn optional(self) -> impl Parser<'a, Option<Output>>
    where
        Self: Sized + 'a,
    {
        poptional(self)
    }

    fn map<NextOutput: Clone + 'a, F: Fn(Output) -> NextOutput + 'a>(
        self,
        f: F,
    ) -> impl Parser<'a, NextOutput>
    where
        Self: Sized + 'a,
        F: Fn(Output) -> NextOutput,
        F: Clone,
    {
        pmap(self, f)
    }

    fn many(self) -> impl Parser<'a, Vec<Token<Output>>>
    where
        Self: Sized + 'a,
    {
        pmany(self)
    }

    fn many1(self) -> impl Parser<'a, Vec<Token<Output>>>
    where
        Self: Sized + 'a,
    {
        self.many().at_least_one()
    }

    fn take_until(self) -> impl Parser<'a, &'a str>
    where
        Self: Sized + 'a,
    {
        ptake_until(self)
    }

    fn any(valid_chars: &'a [char]) -> impl Parser<'a, char>
    where
        Self: Sized + 'a,
    {
        pany(valid_chars)
    }

    fn sep_by<Seperator: Clone + 'a>(
        self,
        separator: impl Parser<'a, Seperator> + 'a,
    ) -> impl Parser<'a, Vec<Token<Output>>>
    where
        Self: Sized + 'a,
    {
        psepby(self, separator)
    }

    fn between<Left: Clone + 'a, Right: Clone + 'a>(
        self,
        parser1: impl Parser<'a, Left> + 'a,
        parser2: impl Parser<'a, Right> + 'a,
    ) -> impl Parser<'a, Output>
    where
        Self: Sized + 'a,
    {
        parser1.then(self.then(parser2)).right().left()
    }

    fn ws(self) -> impl Parser<'a, Output>
    where
        Self: Sized + 'a,
    {
        self.then(pws().many().map(|_| ())).left()
    }
}

pub trait Pair<'a, Left: Clone + 'a, Right: Clone + 'a> {
    fn left(self) -> impl Parser<'a, Left>;
    fn right(self) -> impl Parser<'a, Right>;
}

impl<'a, Left: Clone + 'a, Right: Clone + 'a, T: Parser<'a, (Token<Left>, Token<Right>)> + 'a>
    Pair<'a, Left, Right> for T
{
    fn left(self) -> impl Parser<'a, Left> {
        pleft(self)
    }

    fn right(self) -> impl Parser<'a, Right> {
        pright(self)
    }
}

pub trait Many<'a, Output: Clone + 'a> {
    fn at_least_one(self) -> impl Parser<'a, Vec<Token<Output>>>;
}

impl<'a, Output: Clone + 'a, T: Parser<'a, Vec<Token<Output>>> + 'a> Many<'a, Output> for T {
    fn at_least_one(self) -> impl Parser<'a, Vec<Token<Output>>> {
        p1(self)
    }
}

#[macro_export]
macro_rules! pchoice {
    ($head:expr) => ({
        parser_from_fn(move |input| $head.parse(input))
    });
    ($head:expr, $($tail:expr),*) => ({
        parser_from_fn(
            move |input| {
                let result1 = $head.parse(input.clone());
                result1.or_else(move |error1|{
                    let result = pchoice!($($tail),*).parse(input);
                    result.map_err(|error2| error1 + error2)
                })
            })
    });
}
