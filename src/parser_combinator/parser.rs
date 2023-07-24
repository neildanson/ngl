use crate::{
    parser_combinator::continuation::ContinuationState, parser_combinator::error::*,
    parser_combinator::token::Token,
};

pub type ParseResult<'a, Output> = Result<(Token<Output>, ContinuationState<'a>), Error>;

pub fn pchar<'a>(c: char) -> impl Fn(ContinuationState<'a>) -> ParseResult<char> {
    move |input| {
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
}

pub fn pstring<'a>(value: &'a str) -> impl Fn(ContinuationState<'a>) -> ParseResult<'a, &'a str> {
    move |input| {
        let mut cont = input;
        let mut error = None;
        let mut success = Vec::new();
        for t in value.chars() {
            let parser = pchar(t);
            let result = parser(cont);
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
}

pub fn pthen<'a, T, U>(
    parser1: impl Fn(ContinuationState<'a>) -> ParseResult<T>,
    parser2: impl Fn(ContinuationState<'a>) -> ParseResult<U>,
) -> impl Fn(ContinuationState<'a>) -> ParseResult<(Token<T>, Token<U>)> {
    move |input| {
        let result1 = parser1(input);
        result1.and_then(|(token1, state1)| {
            let result2 = parser2(state1);
            result2.map(|(token2, state2)| {
                let start = token1.start;
                let length = token1.length + token2.length;
                let token = Token::new((token1, token2), start, length);
                (token, state2)
            })
        })
    }
}

pub fn por<'a, T>(
    parser1: impl Fn(ContinuationState<'a>) -> ParseResult<T>,
    parser2: impl Fn(ContinuationState<'a>) -> ParseResult<T>,
) -> impl Fn(ContinuationState<'a>) -> ParseResult<T> {
    move |input| {
        let result1 = parser1(input);
        result1.or_else(|error| {
            let result = parser2(input);
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
}

pub fn poptional<'a, T>(
    parser: impl Fn(ContinuationState<'a>) -> ParseResult<T>,
) -> impl Fn(ContinuationState<'a>) -> ParseResult<Option<T>> {
    move |input| {
        let result1 = parser(input);
        match result1 {
            Ok((token, state)) => Ok((
                Token::new(Some(token.value), token.start, token.length),
                state,
            )),
            Err(_error1) => Ok((Token::new(None, input.position, 0), input)),
        }
    }
}

pub fn pmap<'a, T, U>(
    parser: impl Fn(ContinuationState<'a>) -> ParseResult<T>,
    f: impl Fn(T) -> U,
) -> impl Fn(ContinuationState<'a>) -> ParseResult<U> {
    move |input| {
        let result = parser(input);
        result.map(|(token, state)| {
            let result = f(token.value);
            let token = Token::new(result, token.start, token.length);
            (token, state)
        })
    }
}

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

pub fn pany<'a>(
    valid_chars: &'a [char],
) -> impl Fn(ContinuationState<'a>) -> ParseResult<'a, char> {
    move |input| {
        for c in valid_chars {
            let result = pchar(*c)(input);
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
}

pub fn pmany<'a, T>(
    parser: impl Fn(ContinuationState<'a>) -> ParseResult<T>,
) -> impl Fn(ContinuationState<'a>) -> ParseResult<'a, Vec<Token<T>>> {
    move |input| {
        let mut results = Vec::new();
        let mut cont = input;
        let mut error = None;
        loop {
            let result = parser(cont);
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
            None => Ok((Token::new(results, 0, len), input)),
        }
    }
}

pub fn psepby<'a, T, U, F, FU>(
    parser: F,
    separator: impl Fn(ContinuationState<'a>) -> ParseResult<'a, U>,
) -> impl Fn(ContinuationState<'a>) -> ParseResult<'a, Vec<Token<T>>>
where
    F: Fn() -> FU,
    FU: Fn(ContinuationState<'a>) -> ParseResult<T>,
{
    let parser_combined = pleft(pthen(parser(), separator));
    let parser_many = pmany(parser_combined);
    let parser_many_then = pthen(parser_many, parser());
    let parser = pmap(parser_many_then, |(mut tokens, token)| {
        tokens.value.push(token);
        tokens.value
    });
    parser
}

pub fn pleft<'a, T, U>(
    parser: impl Fn(ContinuationState<'a>) -> ParseResult<(Token<T>, Token<U>)>,
) -> impl Fn(ContinuationState<'a>) -> ParseResult<T> {
    move |input| {
        let result = parser(input);
        result.map(|(token, cont)| {
            let token = token.value.0;
            (token, cont)
        })
    }
}

pub fn pright<'a, T, U>(
    parser: impl Fn(ContinuationState<'a>) -> ParseResult<(Token<T>, Token<U>)>,
) -> impl Fn(ContinuationState<'a>) -> ParseResult<U> {
    move |input| {
        let result = parser(input);
        result.map(|(token, cont)| {
            let token = token.value.1;
            (token, cont)
        })
    }
}

pub fn pbetween<'a, T, U, V>(
    parser1: impl Fn(ContinuationState<'a>) -> ParseResult<T>,
    parser2: impl Fn(ContinuationState<'a>) -> ParseResult<U>,
    parser3: impl Fn(ContinuationState<'a>) -> ParseResult<V>,
) -> impl Fn(ContinuationState<'a>) -> ParseResult<U> {
    let parser = pthen(parser1, pthen(parser2, parser3));
    let parser = pright(parser); //Skip T
    pleft(parser) //Ignore U
}

pub fn p1<'a, T>(
    parser: impl Fn(ContinuationState<'a>) -> ParseResult<'a, Vec<Token<T>>>,
) -> impl Fn(ContinuationState<'a>) -> ParseResult<'a, Vec<Token<T>>> {
    move |input| {
        let result = parser(input);
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
}

pub fn pmany1<'a, T>(
    parser: impl Fn(ContinuationState<'a>) -> ParseResult<T>,
) -> impl Fn(ContinuationState<'a>) -> ParseResult<'a, Vec<Token<T>>> {
    p1(pmany(parser))
}
