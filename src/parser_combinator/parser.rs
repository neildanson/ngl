use crate::{
    parser_combinator::continuation::ContinuationState, parser_combinator::error::*,
    parser_combinator::token::Token,
};

pub type ParseResult<'a, Output> = Result<(Token<Output>, ContinuationState<'a>), Error>;

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
        pmany1(self)
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
        pbetween(parser1, self, parser2)
    }

    /*
    todo - left, right, at_least_one

    fn at_least_one(self) -> impl Parser<'a, Vec<Token<Output>>>
    where
        Self: Sized + 'a,
    {
        p1(self)
    }
    */
}

#[derive(Clone)]
struct ClosureParser<'a, Output, F>
where
    F: Fn(ContinuationState<'a>) -> ParseResult<'a, Output>,
{
    parser: F,
    _phantom: std::marker::PhantomData<&'a Output>,
}

impl<'a, Output: Clone + 'a, F: Clone> ClosureParser<'a, Output, F>
where
    F: Fn(ContinuationState<'a>) -> ParseResult<'a, Output>,
{
    fn new(parser: F) -> impl Parser<'a, Output> {
        parser_from_fn(parser)
    }
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

fn pchar_impl<'a>(c: char, input: ContinuationState<'a>) -> ParseResult<'a, char> {
    let mut chars = input.remaining.chars();
    match chars.next() {
        Some(letter) if letter == c => {
            let parser_state = input.advance(1, letter == '\n');
            Ok((Token::new(c, input.position, 1), parser_state))
        }
        Some(letter) => Err(Error::new(
            c.to_string(),
            letter.to_string(),
            input.position,
            input.line_number,
            input.line_position,
        )),
        None => Err(Error::new(
            c.to_string(),
            "".to_string(),
            input.position,
            input.line_number,
            input.line_position,
        )),
    }
}

fn pstring_impl<'a>(value: &'a str, input: ContinuationState<'a>) -> ParseResult<'a, &'a str> {
    let mut cont = input;
    let mut error = None;
    let mut success = Vec::new();
    for t in value.chars() {
        let result = pchar_impl(t, cont);
        match result {
            Ok((_, new_cont)) => {
                success.push(t);
                cont = new_cont
            }
            Err(err) => {
                let actual = success.iter().collect::<String>() + &err.actual;
                error = Some(Err(Error::new(
                    value.to_string(),
                    actual.to_string(),
                    err.position,
                    err.line_number,
                    err.line_position,
                )));
                break;
            }
        }
    }
    match error {
        Some(err) => err,
        None => Ok((Token::new(value, input.position, value.len()), cont)),
    }
}

fn pthen_impl<'a, T: Clone + 'a, U: Clone + 'a>(
    parser1: impl Parser<'a, T>,
    parser2: impl Parser<'a, U>,
    input: ContinuationState<'a>,
) -> ParseResult<'a, (Token<T>, Token<U>)> {
    let result1 = parser1.parse(input);
    result1.and_then(|(token1, state1)| {
        let result2 = parser2.parse(state1);
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

fn por_impl<'a, T: Clone + 'a>(
    parser1: impl Parser<'a, T>,
    parser2: impl Parser<'a, T>,
    input: ContinuationState<'a>,
) -> ParseResult<'a, T> {
    let result1 = parser1.parse(input);
    result1.or_else(|error| {
        let result = parser2.parse(input);
        match result {
            Ok((token, state)) => Ok((token, state)),
            Err(error2) => {
                let error = Error::new(
                    error.expected + " or " + &error2.expected,
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

fn poptional_impl<'a, T: Clone + 'a>(
    parser: impl Parser<'a, T>,
    input: ContinuationState<'a>,
) -> ParseResult<'a, Option<T>> {
    let result1 = parser.parse(input);
    match result1 {
        Ok((token, state)) => Ok((
            Token::new(Some(token.value), token.start, token.length),
            state,
        )),
        Err(_error1) => Ok((Token::new(None, input.position, 0), input)),
    }
}

fn pmap_impl<'a, T: Clone + 'a, U, F>(
    parser: impl Parser<'a, T>,
    f: F,
    input: ContinuationState<'a>,
) -> ParseResult<'a, U>
where
    F: Fn(T) -> U,
    F: Clone,
{
    let result = parser.parse(input);
    result.map(|(token, state)| {
        let result = f(token.value);
        let token = Token::new(result, token.start, token.length);
        (token, state)
    })
}

fn pany_impl<'a>(valid_chars: &[char], input: ContinuationState<'a>) -> ParseResult<'a, char> {
    for c in valid_chars.iter() {
        let result = pchar_impl(*c, input);
        match result {
            Ok((token, state)) => return Ok((token, state)),
            Err(_) => continue,
        }
    }

    let valid_chars_length = valid_chars.len();
    let error = if valid_chars_length >= 2 {
        let first = valid_chars
            .iter()
            .take(valid_chars.len() - 1)
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        first + " or " + &valid_chars.last().unwrap().to_string()
    } else if valid_chars_length == 1 {
        valid_chars.first().unwrap().to_string()
    } else {
        "".to_string() //TODO - this should never happen
    };

    let actual = input.remaining.chars().next().unwrap_or(' ').to_string();

    Err(Error::new(
        error,
        actual,
        input.position,
        input.line_number,
        input.line_position,
    ))
}

fn pmany_impl<'a, T: Clone + 'a>(
    parser: impl Parser<'a, T>,
    input: ContinuationState<'a>,
) -> ParseResult<'a, Vec<Token<T>>> {
    let mut results = Vec::new();
    let mut cont = input;
    let mut error = None;
    while error.is_none() {
        let result = parser.parse(cont);
        match result {
            Ok((token, state)) => {
                results.push(token);
                cont = state;
            }
            Err(err) => {
                error = Some(err);
            }
        }
    }

    let len = results.len();
    match error {
        Some(_) => Ok((Token::new(results, input.position, len), cont)),
        None => Ok((Token::new(results, input.position, len), input)),
    }
}

fn pleft_impl<'a, T: Clone + 'a, U: Clone + 'a>(
    parser: impl Parser<'a, (Token<T>, Token<U>)>,
    input: ContinuationState<'a>,
) -> ParseResult<'a, T> {
    let result = parser.parse(input);
    result.map(|(token, cont)| {
        let token = token.value.0;
        (token, cont)
    })
}

fn pright_impl<'a, T: Clone + 'a, U: Clone + 'a>(
    parser: impl Parser<'a, (Token<T>, Token<U>)>,
    input: ContinuationState<'a>,
) -> ParseResult<'a, U> {
    let result = parser.parse(input);
    result.map(|(token, cont)| {
        let token = token.value.1;
        (token, cont)
    })
}

fn p1_impl<'a, T: Clone + 'a>(
    parser: impl Parser<'a, Vec<Token<T>>>,
    input: ContinuationState<'a>,
) -> ParseResult<'a, Vec<Token<T>>> {
    let result = parser.parse(input);
    match result {
        Ok((token, cont)) => {
            if token.length == 0 {
                Err(Error::new(
                    "1 or more".to_string(),
                    cont.remaining.to_string(),
                    input.position,
                    input.line_number,
                    input.line_position,
                ))
            } else {
                Ok((token, cont))
            }
        }
        Err(err) => Err(err),
    }
}

fn pchoice_impl<'a, T: Clone + 'a>(
    parsers: Vec<impl Parser<'a, T>>,
    input: ContinuationState<'a>,
) -> ParseResult<'a, T> {
    let mut errors = Vec::new();
    for parser in parsers.iter() {
        let result = parser.parse(input);
        match result {
            Ok((token, cont)) => return Ok((token, cont)),
            Err(err) => errors.push(err),
        }
    }

    let mut error = errors.remove(0);
    for err in errors.into_iter() {
        error = error + err;
    }

    Err(error)
}

//TODO deal with case where string is never termianted
fn ptakeuntil_impl<'a, T: Clone + 'a>(
    until: impl Parser<'a, T>,
    start: Option<ContinuationState<'a>>,
    input: ContinuationState<'a>,
) -> ParseResult<'a, &'a str> {
    let result = until.parse(input);
    let start = start.unwrap_or_else(|| input);
    match result {
        Ok((_, cont)) => {
            let len = cont.position - start.position - 1;
            return Ok((
                Token::new(&start.remaining[0..len], start.position, len),
                cont,
            ));
        }
        Err(_) => {
            let cont = input.advance(1, false); //TODO line advances
            return ptakeuntil_impl(until, Some(start), cont);
        }
    }
}

#[derive(Clone)]
struct CharParser {
    value: char,
}

impl<'a> Parser<'a, char> for CharParser {
    fn parse(&self, input: ContinuationState<'a>) -> ParseResult<'a, char> {
        pchar_impl(self.value, input)
    }
}

//TODO - can I make these using a macro????
pub fn pchar<'a>(value: char) -> impl Parser<'a, char> {
    CharParser { value: value }
}

#[derive(Clone)]
struct StringParser<'a> {
    value: &'a str,
}

impl<'a> Parser<'a, &'a str> for StringParser<'a> {
    fn parse(&self, input: ContinuationState<'a>) -> ParseResult<'a, &'a str> {
        pstring_impl(self.value, input)
    }
}

pub fn pstring<'a>(value: &'a str) -> impl Parser<'a, &str> {
    StringParser { value }
}

#[derive(Clone)]
struct ThenParser<'a, T: Clone + 'a, U: Clone + 'a, P1: Parser<'a, T>, P2: Parser<'a, U>> {
    parser1: P1,
    parser2: P2,
    _phantom: std::marker::PhantomData<&'a (T, U)>,
}

impl<'a, T: Clone, U: Clone, P1, P2> Parser<'a, (Token<T>, Token<U>)>
    for ThenParser<'a, T, U, P1, P2>
where
    P1: Parser<'a, T>,
    P2: Parser<'a, U>,
{
    fn parse(&self, input: ContinuationState<'a>) -> ParseResult<'a, (Token<T>, Token<U>)> {
        pthen_impl(self.parser1.clone(), self.parser2.clone(), input)
    }
}

fn pthen<'a, T: Clone + 'a, U: Clone + 'a>(
    parser1: impl Parser<'a, T> + 'a,
    parser2: impl Parser<'a, U> + 'a,
) -> impl Parser<'a, (Token<T>, Token<U>)> {
    ThenParser {
        parser1,
        parser2,
        _phantom: std::marker::PhantomData,
    }
}

#[derive(Clone)]
struct OrParser<'a, T: Clone + 'a, P1: Parser<'a, T>, P2: Parser<'a, T>> {
    parser1: P1,
    parser2: P2,
    _phantom: std::marker::PhantomData<&'a T>,
}

impl<'a, T: Clone, P1, P2> Parser<'a, T> for OrParser<'a, T, P1, P2>
where
    P1: Parser<'a, T>,
    P2: Parser<'a, T>,
{
    fn parse(&self, input: ContinuationState<'a>) -> ParseResult<'a, T> {
        por_impl(self.parser1.clone(), self.parser2.clone(), input)
    }
}

fn por<'a, T: Clone + 'a>(
    parser1: impl Parser<'a, T> + 'a,
    parser2: impl Parser<'a, T> + 'a,
) -> impl Parser<'a, T> {
    OrParser {
        parser1,
        parser2,
        _phantom: std::marker::PhantomData,
    }
}

#[derive(Clone)]
struct OptionalParser<'a, T: Clone + 'a, P: Parser<'a, T>> {
    parser: P,
    _phantom: std::marker::PhantomData<&'a T>,
}

impl<'a, T: Clone, P> Parser<'a, Option<T>> for OptionalParser<'a, T, P>
where
    P: Parser<'a, T>,
{
    fn parse(&self, input: ContinuationState<'a>) -> ParseResult<'a, Option<T>> {
        poptional_impl(self.parser.clone(), input)
    }
}

fn poptional<'a, T: Clone + 'a>(parser: impl Parser<'a, T> + 'a) -> impl Parser<'a, Option<T>> {
    OptionalParser {
        parser,
        _phantom: std::marker::PhantomData,
    }
}

pub fn pany(valid_chars: &[char]) -> impl Parser<char> {
    ClosureParser::new(move |input| pany_impl(valid_chars, input))
}

fn pmap<'a, T: Clone + 'a, U: Clone + 'a, F>(
    parser: impl Parser<'a, T> + 'a,
    f: F,
) -> impl Parser<'a, U>
where
    F: Fn(T) -> U,
    F: Clone + 'a,
{
    ClosureParser::new(move |input| pmap_impl(parser.clone(), f.clone(), input))
}

fn pmany<'a, T: Clone + 'a>(parser: impl Parser<'a, T> + 'a) -> impl Parser<'a, Vec<Token<T>>> {
    ClosureParser::new(move |input| pmany_impl(parser.clone(), input))
}

pub fn pleft<'a, T: Clone + 'a, U: Clone + 'a>(
    parser: impl Parser<'a, (Token<T>, Token<U>)> + 'a,
) -> impl Parser<'a, T> {
    ClosureParser::new(move |input| pleft_impl(parser.clone(), input))
}

pub fn pright<'a, T: Clone + 'a, U: Clone + 'a>(
    parser: impl Parser<'a, (Token<T>, Token<U>)> + 'a,
) -> impl Parser<'a, U> {
    ClosureParser::new(move |input| pright_impl(parser.clone(), input))
}

fn pbetween<'a, T: Clone + 'a, U: Clone + 'a, V: Clone + 'a>(
    parser1: impl Parser<'a, T> + 'a,
    parser2: impl Parser<'a, U> + 'a,
    parser3: impl Parser<'a, V> + 'a,
) -> impl Parser<'a, U> {
    let parser = pthen(parser1, pthen(parser2, parser3));
    let parser = pright(parser); //Skip T
    pleft(parser) //Ignore U
}

pub fn p1<'a, T: Clone + 'a>(
    parser: impl Parser<'a, Vec<Token<T>>> + 'a,
) -> impl Parser<'a, Vec<Token<T>>> {
    ClosureParser::new(move |input| p1_impl(parser.clone(), input))
}

fn psepby<'a, T: Clone + 'a, U: Clone + 'a>(
    parser: impl Parser<'a, T> + 'a,
    separator: impl Parser<'a, U> + 'a,
) -> impl Parser<'a, Vec<Token<T>>> {
    let parser_combined = pleft(pthen(parser.clone(), separator));
    let parser_many = pmany(parser_combined);
    let parser_many_then = pthen(parser_many, parser);
    let parser = pmap(parser_many_then, |(mut tokens, token)| {
        tokens.value.push(token);
        tokens.value
    });
    parser
}

fn pmany1<'a, T: Clone + 'a>(parser: impl Parser<'a, T> + 'a) -> impl Parser<'a, Vec<Token<T>>> {
    p1(pmany(parser))
}

pub fn pchoice<'a, T: Clone + 'a>(parsers: Vec<impl Parser<'a, T>>) -> impl Parser<'a, T> {
    ClosureParser::new(move |input| pchoice_impl(parsers.clone(), input))
}

fn ptake_until<'a, T: Clone + 'a>(until: impl Parser<'a, T>) -> impl Parser<'a, &'a str> {
    ClosureParser::new(move |input| ptakeuntil_impl(until.clone(), None, input))
}

#[macro_export]
macro_rules! pchoice {
    ($head:expr) => ({
        parser_from_fn(move |input| $head.parse(input))
    });
    ($head:expr, $($tail:expr),*) => ({
        parser_from_fn(
            move |input| {
                let result1 = $head.parse(input);
                result1.or_else(|error1|{
                    let result = pchoice!($($tail),*).parse(input);
                    result.map_err(|error2| error1 + error2)
                })
            })
    });
}
