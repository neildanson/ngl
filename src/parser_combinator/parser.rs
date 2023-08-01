use crate::{
    parser_combinator::continuation::ContinuationState, parser_combinator::error::*,
    parser_combinator::token::Token,
};

pub type ParseResult<'a, Output> = Result<(Token<Output>, ContinuationState<'a>), Error>;

pub trait Parser<'a>: Clone {
    type Output;
    fn parse(&self, input: ContinuationState<'a>) -> ParseResult<'a, Self::Output>;
}

#[derive(Clone)]
struct ClosureParser<'a, Output, F>
where
    F: Fn(ContinuationState<'a>) -> ParseResult<'a, Output>,
{
    parser: F,
    _phantom: std::marker::PhantomData<&'a Output>,
}

impl<'a, Output, F> ClosureParser<'a, Output, F>
where
    F: Fn(ContinuationState<'a>) -> ParseResult<'a, Output>,
{
    pub fn new(parser: F) -> Self {
        ClosureParser {
            parser,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<'a, Output: Clone, F> Parser<'a> for ClosureParser<'a, Output, F>
where
    F: Fn(ContinuationState<'a>) -> ParseResult<'a, Output> + Clone,
{
    type Output = Output;
    fn parse(&self, input: ContinuationState<'a>) -> ParseResult<'a, Output> {
        (self.parser)(input)
    }
}

fn pchar_impl<'a>(c: char, input: ContinuationState<'a>) -> ParseResult<char> {
    let mut chars = input.remaining.chars();
    match chars.next() {
        Some(letter) if letter == c => {
            let new_line = if letter == '\n' { 1 } else { 0 };
            let parser_state = input.advance(1, new_line);
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
        None => Ok((Token::new(value, input.position, value.len()), cont)), //This seems to work, but I dont know why!
    }
}

fn pthen_impl<'a, T, U>(
    parser1: impl Parser<'a, Output = T>,
    parser2: impl Parser<'a, Output = U>,
    input: ContinuationState<'a>,
) -> ParseResult<'a, (Token<T>, Token<U>)> {
    let result1 = parser1.parse(input);
    result1.and_then(|(token1, state1)| {
        let result2 = parser2.parse(state1);
        result2.map(|(token2, state2)| {
            let start = token1.start;
            let length = token1.length + token2.length;
            let token = Token::new((token1, token2), start, length);
            (token, state2)
        })
    })
}

pub fn por_impl<'a, T>(
    parser1: impl Parser<'a, Output = T>,
    parser2: impl Parser<'a, Output = T>,
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

fn poptional_impl<'a, T>(
    parser: impl Parser<'a, Output = T>,
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

fn pmap_impl<'a, T, U, F>(
    parser: impl Parser<'a, Output = T>,
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

fn pany_impl<'a>(valid_chars: &'a [char], input: ContinuationState<'a>) -> ParseResult<'a, char> {
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
        valid_chars.iter().next().unwrap().to_string()
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

fn pmany_impl<'a, T>(
    parser: impl Parser<'a, Output = T>,
    input: ContinuationState<'a>,
) -> ParseResult<'a, Vec<Token<T>>> {
    let mut results = Vec::new();
    let mut cont = input;
    let mut error = None;
    loop {
        let result = parser.parse(cont);
        match result {
            Ok((token, state)) => {
                results.push(token);
                cont = state;
            }
            Err(err) => {
                error = Some(err);
                break;
            }
        }
    }

    let len = results.len();
    match error {
        Some(_) => Ok((Token::new(results, input.position, len), cont)),
        None => Ok((Token::new(results, input.position, len), input)),
    }
}

fn pleft_impl<'a, T, U>(
    parser: impl Parser<'a, Output = (Token<T>, Token<U>)>,
    input: ContinuationState<'a>,
) -> ParseResult<'a, T> {
    let result = parser.parse(input);
    result.map(|(token, cont)| {
        let token = token.value.0;
        (token, cont)
    })
}

fn pright_impl<'a, T, U>(
    parser: impl Parser<'a, Output = (Token<T>, Token<U>)>,
    input: ContinuationState<'a>,
) -> ParseResult<'a, U> {
    let result = parser.parse(input);
    result.map(|(token, cont)| {
        let token = token.value.1;
        (token, cont)
    })
}

fn p1_impl<'a, T>(
    parser: impl Parser<'a, Output = Vec<Token<T>>>,
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

//TODO - can I make these using a macro????
pub fn pchar<'a>(value: char) -> impl Parser<'a, Output = char> {
    ClosureParser::new(move |input| pchar_impl(value, input))
}

pub fn pstring<'a>(value: &'a str) -> impl Parser<'a, Output = &'a str> {
    ClosureParser::new(move |input| pstring_impl(value, input))
}

pub fn pthen<'a, T: Clone + 'a, U: Clone + 'a>(
    parser1: impl Parser<'a, Output = T> + 'a,
    parser2: impl Parser<'a, Output = U> + 'a,
) -> impl Parser<'a, Output = (Token<T>, Token<U>)> {
    ClosureParser::new(move |input| pthen_impl(parser1.clone(), parser2.clone(), input))
}

pub fn por<'a, T: Clone + 'a>(
    parser1: impl Parser<'a, Output = T> + 'a,
    parser2: impl Parser<'a, Output = T> + 'a,
) -> impl Parser<'a, Output = T> {
    ClosureParser::new(move |input| por_impl(parser1.clone(), parser2.clone(), input))
}

pub fn poptional<'a, T: Clone + 'a>(
    parser: impl Parser<'a, Output = T> + 'a,
) -> impl Parser<'a, Output = Option<T>> {
    ClosureParser::new(move |input| poptional_impl(parser.clone(), input))
}

pub fn pany<'a>(valid_chars: &'a [char]) -> impl Parser<'a, Output = char> {
    ClosureParser::new(move |input| pany_impl(valid_chars, input))
}

pub fn pmap<'a, T: 'a, U: Clone + 'a, F>(
    parser: impl Parser<'a, Output = T> + 'a,
    f: F,
) -> impl Parser<'a, Output = U>
where
    F: Fn(T) -> U,
    F: Clone + 'a,
{
    ClosureParser::new(move |input| pmap_impl(parser.clone(), f.clone(), input))
}

pub fn pmany<'a, T: Clone + 'a>(
    parser: impl Parser<'a, Output = T> + 'a,
) -> impl Parser<'a, Output = Vec<Token<T>>> {
    ClosureParser::new(move |input| pmany_impl(parser.clone(), input))
}

pub fn pleft<'a, T: Clone + 'a, U: Clone + 'a>(
    parser: impl Parser<'a, Output = (Token<T>, Token<U>)> + 'a,
) -> impl Parser<'a, Output = T> {
    ClosureParser::new(move |input| pleft_impl(parser.clone(), input))
}

pub fn pright<'a, T: Clone + 'a, U: Clone + 'a>(
    parser: impl Parser<'a, Output = (Token<T>, Token<U>)> + 'a,
) -> impl Parser<'a, Output = U> {
    ClosureParser::new(move |input| pright_impl(parser.clone(), input))
}

pub fn pbetween<'a, T: Clone + 'a, U: Clone + 'a, V: Clone + 'a>(
    parser1: impl Parser<'a, Output = T> + 'a,
    parser2: impl Parser<'a, Output = U> + 'a,
    parser3: impl Parser<'a, Output = V> + 'a,
) -> impl Parser<'a, Output = U> {
    let parser = pthen(parser1, pthen(parser2, parser3));
    let parser = pright(parser); //Skip T
    pleft(parser) //Ignore U
}

pub fn p1<'a, T: Clone + 'a>(
    parser: impl Parser<'a, Output = Vec<Token<T>>> + 'a,
) -> impl Parser<'a, Output = Vec<Token<T>>> {
    ClosureParser::new(move |input| p1_impl(parser.clone(), input))
}

pub fn psepby<'a, T: Clone + 'a, U: Clone + 'a>(
    parser: impl Parser<'a, Output = T> + 'a,
    separator: impl Parser<'a, Output = U> + 'a,
) -> impl Parser<'a, Output = Vec<Token<T>>> {
    let parser_combined = pleft(pthen(parser.clone(), separator));
    let parser_many = pmany(parser_combined);
    let parser_many_then = pthen(parser_many, parser);
    let parser = pmap(parser_many_then, |(mut tokens, token)| {
        tokens.value.push(token);
        tokens.value
    });
    parser
}

pub fn pmany1<'a, T: Clone + 'a>(
    parser: impl Parser<'a, Output = T> + 'a,
) -> impl Parser<'a, Output = Vec<Token<T>>> {
    p1(pmany(parser))
}

/*
#[macro_export]
macro_rules! pchoice {
    ($head:expr) => ({
        move |input| $head(input) //TODO - we should accumulate the errors for choice (ie "a" or "b" )
    });
    ($head:expr, $($tail:expr),*) => ({
        move |input| {
            let result1 = $head(input);
            result1.or_else(|error1|{
                let result = pchoice!($($tail),*)(input);
                result.map_err(|error2| error1 + error2)
            })
        }});
}
 */
