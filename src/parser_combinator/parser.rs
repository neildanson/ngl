use std::ops::RangeInclusive;

use crate::{
    parser_combinator::continuation::ContinuationState, parser_combinator::error::*,
    parser_combinator::token::Token,
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

    fn ws(self) -> impl Parser<'a, Output>
    where
        Self: Sized + 'a,
    {
        pleft(self.then(pws_many()))
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

fn pchar_impl(c: char, input: ContinuationState<'_>) -> ParseResult<'_, char> {
    let mut chars = input.remaining.chars();
    match chars.next() {
        Some(letter) if letter == c => {
            let parser_state = input.advance(1, letter == '\n');
            Ok((Token::new(c, input.position, 1), parser_state))
        }
        Some(letter) => Err(Error::new(
            c.into(),
            letter.to_string(),
            input.position,
            input.line_number,
            input.line_position,
        )),
        None => Err(Error::new(
            c.into(),
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
    for t in value.chars() {
        let result = pchar_impl(t, cont);
        match result {
            Ok((_, new_cont)) => cont = new_cont,
            Err(err) => {
                let length = err.position - input.position + 1;
                let actual = if input.remaining.len() < length {
                    input.remaining[0..].to_string()
                } else {
                    input.remaining[0..length].to_string()
                };

                error = Some(Err(Error::new(
                    value.into(),
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

fn pthen_impl<'a, Left: Clone + 'a, Right: Clone + 'a>(
    parser1: &impl Parser<'a, Left>,
    parser2: &impl Parser<'a, Right>,
    input: ContinuationState<'a>,
) -> ParseResult<'a, (Token<Left>, Token<Right>)> {
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

fn por_impl<'a, Output: Clone + 'a>(
    parser1: &impl Parser<'a, Output>,
    parser2: &impl Parser<'a, Output>,
    input: ContinuationState<'a>,
) -> ParseResult<'a, Output> {
    let result1 = parser1.parse(input);
    result1.or_else(|error| {
        let result = parser2.parse(input);
        match result {
            Ok((token, state)) => Ok((token, state)),
            Err(error2) => {
                let error = Error::new(
                    error.expected + error2.expected,
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

fn poptional_impl<'a, Output: Clone + 'a>(
    parser: &impl Parser<'a, Output>,
    input: ContinuationState<'a>,
) -> ParseResult<'a, Option<Output>> {
    let result1 = parser.parse(input);
    match result1 {
        Ok((token, state)) => Ok((
            Token::new(Some(token.value), token.start, token.length),
            state,
        )),
        Err(_error1) => Ok((Token::new(None, input.position, 0), input)),
    }
}

fn pmap_impl<'a, Input: Clone + 'a, Output, F>(
    parser: &impl Parser<'a, Input>,
    f: &F,
    input: ContinuationState<'a>,
) -> ParseResult<'a, Output>
where
    F: Fn(Input) -> Output,
    F: Clone,
{
    let result = parser.parse(input);
    result.map(|(token, state)| {
        let result = f(token.value);
        let token = Token::new(result, token.start, token.length);
        (token, state)
    })
}

fn pws_impl(input: ContinuationState<'_>) -> ParseResult<'_, ()> {
    let next_char = input.remaining.chars().next();
    if let Some(next_char) = next_char {
        if next_char.is_whitespace() {
            let parser_state = input.advance(1, next_char == '\n');
            return Ok((Token::new((), input.position, 1), parser_state));
        }
    }

    let actual = next_char.unwrap_or(' ').to_string();
    Err(Error::new(
        Expected::Char(' '),
        actual,
        input.position,
        input.line_number,
        input.line_position,
    ))
}

fn panyrange_impl<'a>(
    valid_chars: &RangeInclusive<char>,
    input: ContinuationState<'a>,
) -> ParseResult<'a, char> {
    let next_char = input.remaining.chars().next();
    if let Some(next_char) = next_char {
        if valid_chars.contains(&next_char) {
            let parser_state = input.advance(1, next_char == '\n');
            return Ok((Token::new(next_char, input.position, 1), parser_state));
        }
    }

    let actual = next_char.unwrap_or(' ').to_string();

    Err(Error::new(
        valid_chars.clone().into(),
        actual,
        input.position,
        input.line_number,
        input.line_position,
    ))
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
    /*let error = if valid_chars_length >= 2 {
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
    };*/

    let actual = input.remaining.chars().next().unwrap_or(' ').to_string();

    Err(Error::new(
        "error".into(),
        actual,
        input.position,
        input.line_number,
        input.line_position,
    ))
}

fn pmany_impl<'a, Output: Clone + 'a>(
    parser: &impl Parser<'a, Output>,
    input: ContinuationState<'a>,
) -> ParseResult<'a, Vec<Token<Output>>> {
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

fn pleft_impl<'a, Left: Clone + 'a, Right: Clone + 'a>(
    parser: &impl Parser<'a, (Token<Left>, Token<Right>)>,
    input: ContinuationState<'a>,
) -> ParseResult<'a, Left> {
    let result = parser.parse(input);
    result.map(|(token, cont)| {
        let token = token.value.0;
        (token, cont)
    })
}

fn pright_impl<'a, Left: Clone + 'a, Right: Clone + 'a>(
    parser: &impl Parser<'a, (Token<Left>, Token<Right>)>,
    input: ContinuationState<'a>,
) -> ParseResult<'a, Right> {
    let result = parser.parse(input);
    result.map(|(token, cont)| {
        let token = token.value.1;
        (token, cont)
    })
}

fn p1_impl<'a, Output: Clone + 'a>(
    parser: &impl Parser<'a, Vec<Token<Output>>>,
    input: ContinuationState<'a>,
) -> ParseResult<'a, Vec<Token<Output>>> {
    let result = parser.parse(input);
    match result {
        Ok((token, cont)) => {
            if token.length == 0 {
                Err(Error::new(
                    "1 or more".into(),
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

fn pchoice_impl<'a, Output: Clone + 'a>(
    parsers: &[impl Parser<'a, Output>],
    input: ContinuationState<'a>,
) -> ParseResult<'a, Output> {
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
fn ptakeuntil_impl<'a, Until: Clone + 'a>(
    until: &impl Parser<'a, Until>,
    start: Option<ContinuationState<'a>>,
    input: ContinuationState<'a>,
) -> ParseResult<'a, &'a str> {
    let result = until.parse(input);
    let start = start.unwrap_or(input);
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
    CharParser { value }
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

pub fn pstring(value: &str) -> impl Parser<'_, &str> {
    StringParser { value }
}

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
        pthen_impl(&self.parser1, &self.parser2, input)
    }
}

fn pthen<'a, Left: Clone + 'a, Right: Clone + 'a>(
    parser1: impl Parser<'a, Left> + 'a,
    parser2: impl Parser<'a, Right> + 'a,
) -> impl Parser<'a, (Token<Left>, Token<Right>)> {
    ThenParser {
        parser1,
        parser2,
        _phantom: std::marker::PhantomData,
    }
}

#[derive(Clone)]
struct OrParser<'a, Output: Clone + 'a, P1: Parser<'a, Output>, P2: Parser<'a, Output>> {
    parser1: P1,
    parser2: P2,
    _phantom: std::marker::PhantomData<&'a Output>,
}

impl<'a, Output: Clone, P1, P2> Parser<'a, Output> for OrParser<'a, Output, P1, P2>
where
    P1: Parser<'a, Output>,
    P2: Parser<'a, Output>,
{
    fn parse(&self, input: ContinuationState<'a>) -> ParseResult<'a, Output> {
        por_impl(&self.parser1, &self.parser2, input)
    }
}

fn por<'a, Output: Clone + 'a>(
    parser1: impl Parser<'a, Output> + 'a,
    parser2: impl Parser<'a, Output> + 'a,
) -> impl Parser<'a, Output> {
    OrParser {
        parser1,
        parser2,
        _phantom: std::marker::PhantomData,
    }
}

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
        poptional_impl(&self.parser, input)
    }
}

fn poptional<'a, Output: Clone + 'a>(
    parser: impl Parser<'a, Output> + 'a,
) -> impl Parser<'a, Option<Output>> {
    OptionalParser {
        parser,
        _phantom: std::marker::PhantomData,
    }
}

#[derive(Clone)]
struct AnyParser<'a> {
    valid_chars: &'a [char],
}

impl<'a> Parser<'a, char> for AnyParser<'a> {
    fn parse(&self, input: ContinuationState<'a>) -> ParseResult<'a, char> {
        pany_impl(self.valid_chars, input)
    }
}

pub fn pany(valid_chars: &[char]) -> impl Parser<char> {
    AnyParser { valid_chars }
}

#[derive(Clone)]
struct AnyRangeParser {
    valid_chars: RangeInclusive<char>,
}

impl<'a> Parser<'a, char> for AnyRangeParser {
    fn parse(&self, input: ContinuationState<'a>) -> ParseResult<'a, char> {
        panyrange_impl(&self.valid_chars, input)
    }
}

pub fn pany_range<'a>(valid_chars: RangeInclusive<char>) -> impl Parser<'a, char> {
    AnyRangeParser { valid_chars }
}

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
        pmap_impl(&self.parser, &self.f, input)
    }
}

fn pmap<'a, Input: Clone + 'a, Output: Clone + 'a, F>(
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

#[derive(Clone)]
struct ManyParser<'a, Output: Clone + 'a, P: Parser<'a, Output>> {
    parser: P,
    _phantom: std::marker::PhantomData<&'a Output>,
}

impl<'a, Output: Clone + 'a, P> Parser<'a, Vec<Token<Output>>> for ManyParser<'a, Output, P>
where
    P: Parser<'a, Output>,
{
    fn parse(&self, input: ContinuationState<'a>) -> ParseResult<'a, Vec<Token<Output>>> {
        pmany_impl(&self.parser, input)
    }
}

fn pmany<'a, Output: Clone + 'a>(
    parser: impl Parser<'a, Output> + 'a,
) -> impl Parser<'a, Vec<Token<Output>>> {
    {
        ManyParser {
            parser,
            _phantom: std::marker::PhantomData,
        }
    }
}

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
        pleft_impl(&self.parser, input)
    }
}

fn pleft<'a, Left: Clone + 'a, Right: Clone + 'a>(
    parser: impl Parser<'a, (Token<Left>, Token<Right>)> + 'a,
) -> impl Parser<'a, Left> {
    LeftParser {
        parser,
        _phantom: std::marker::PhantomData,
    }
}

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
        pright_impl(&self.parser, input)
    }
}

fn pright<'a, Left: Clone + 'a, Right: Clone + 'a>(
    parser: impl Parser<'a, (Token<Left>, Token<Right>)> + 'a,
) -> impl Parser<'a, Right> {
    RightParser {
        parser,
        _phantom: std::marker::PhantomData,
    }
}

fn pbetween<'a, Left: Clone + 'a, Output: Clone + 'a, Right: Clone + 'a>(
    parser1: impl Parser<'a, Left> + 'a,
    parser2: impl Parser<'a, Output> + 'a,
    parser3: impl Parser<'a, Right> + 'a,
) -> impl Parser<'a, Output> {
    parser1.then(parser2.then(parser3)).right().left()
}

#[derive(Clone)]
struct OneParser<'a, P, Output: Clone>
where
    P: Parser<'a, Vec<Token<Output>>>,
{
    parser: P,
    _phantom: std::marker::PhantomData<&'a Output>,
}

impl<'a, P, Output: Clone> Parser<'a, Vec<Token<Output>>> for OneParser<'a, P, Output>
where
    P: Parser<'a, Vec<Token<Output>>>,
{
    fn parse(&self, input: ContinuationState<'a>) -> ParseResult<'a, Vec<Token<Output>>> {
        p1_impl(&self.parser, input)
    }
}

fn p1<'a, Output: Clone + 'a>(
    parser: impl Parser<'a, Vec<Token<Output>>> + 'a,
) -> impl Parser<'a, Vec<Token<Output>>> {
    {
        OneParser {
            parser,
            _phantom: std::marker::PhantomData,
        }
    }
}

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

fn psepby<'a, Output: Clone + 'a, Seperator: Clone + 'a>(
    parser: impl Parser<'a, Output> + 'a,
    separator: impl Parser<'a, Seperator> + 'a,
) -> impl Parser<'a, Vec<Token<Output>>> {
    SepByParser {
        parser,
        separator,
        _phantom: std::marker::PhantomData,
    }
}

fn pmany1<'a, Output: Clone + 'a>(
    parser: impl Parser<'a, Output> + 'a,
) -> impl Parser<'a, Vec<Token<Output>>> {
    parser.many().at_least_one()
}

#[derive(Clone)]
struct ChoiceParser<'a, P, Output: Clone + 'a>
where
    P: Parser<'a, Output>,
{
    parsers: Vec<P>,
    _phantom: std::marker::PhantomData<&'a Output>,
}

impl<'a, P, Output: Clone> Parser<'a, Output> for ChoiceParser<'a, P, Output>
where
    P: Parser<'a, Output>,
{
    fn parse(&self, input: ContinuationState<'a>) -> ParseResult<'a, Output> {
        pchoice_impl(&self.parsers, input)
    }
}

pub fn pchoice<'a, Output: Clone + 'a>(
    parsers: Vec<impl Parser<'a, Output>>,
) -> impl Parser<'a, Output> {
    ChoiceParser {
        parsers,
        _phantom: std::marker::PhantomData,
    }
}

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

fn ptake_until<'a, Until: Clone + 'a>(until: impl Parser<'a, Until>) -> impl Parser<'a, &'a str> {
    TakeUntilParser {
        until,
        _phantom: std::marker::PhantomData,
    }
}

#[derive(Clone)]
struct WhitespaceParser;

impl<'a> Parser<'a, ()> for WhitespaceParser {
    fn parse(&self, input: ContinuationState<'a>) -> ParseResult<'a, ()> {
        pws_impl(input)
    }
}

pub fn pws<'a>() -> impl Parser<'a, ()> {
    WhitespaceParser
}

fn pws_many<'a>() -> impl Parser<'a, ()> {
    pws().many().map(|_| ())
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
