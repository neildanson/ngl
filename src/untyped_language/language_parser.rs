use crate::parser_combinator::*;

use super::*;

pub(crate) fn pint<'a>() -> impl Parser<'a, Output = Value> {
    let any_number = pany(&['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']);
    let many_numbers = pmany1(any_number);
    let number_parser = pthen(poptional(pchar('-')), many_numbers);
    let pnumber = pmap(number_parser, move |(negate, value)| {
        let string: String = value.value.into_iter().map(|c| c.value).collect();
        let number = string.parse::<i32>().unwrap();
        match negate.value {
            Some(_) => -number,
            None => number,
        }
    });
    pmap(pnumber, |n| Value::Number(n))
}

fn pbool<'a>() -> impl Parser<'a, Output = Value> {
    let ptrue = pmap(pstring("true"), |_| true);
    let pfalse = pmap(pstring("false"), |_| false);
    pmap(por(ptrue, pfalse), |b| Value::Bool(b))
}

fn pvalue<'a>() -> impl Parser<'a, Output = Value> {
    por(pint(), pbool())
}

pub fn pidentifier<'a>() -> impl Parser<'a, Output = String> {
    let ident = pmany(pany(&[
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
        's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    ]));
    pmap(ident, |ident| {
        let string: String = ident.into_iter().map(|c| c.value).collect();
        string
    })
}

pub fn pws<'a>() -> impl Parser<'a, Output = Vec<Token<char>>> {
    pmany(pany(&[' ', '\n', '\t', '\r']))
}

pub fn pparam<'a>() -> impl Parser<'a, Output = Parameter> {
    let param_binding = pleft(pthen(pidentifier(), pws()));
    let param_binding = pleft(pthen(param_binding, pchar(':')));
    let param_binding = pleft(pthen(param_binding, pws()));
    let param_binding = pthen(param_binding, pidentifier());
    let param_binding = pleft(pthen(param_binding, pws()));
    pmap(param_binding, |(name, type_)| {
        Parameter(name.value, type_.value)
    })
}

pub fn plet<'a>() -> impl Parser<'a, Output = ()> {
    let let_binding = pleft(pthen(pstring("let"), pws()));
    let let_binding = pright(pthen(let_binding, pidentifier()));
    let let_binding = pleft(pthen(let_binding, pws()));
    let let_binding = pleft(pthen(let_binding, pchar('=')));
    let let_binding = pleft(pthen(let_binding, pws()));
    let let_binding = pthen(let_binding, pvalue());
    let let_binding = pleft(pthen(let_binding, pws()));
    let let_binding = pleft(pthen(let_binding, pchar(';')));
    let let_binding = pleft(pthen(let_binding, pws()));
    pmap(let_binding, |_| ())
}

pub fn pfun<'a>() -> impl Parser<'a, Output = Fun> {
    let fun_binding = pleft(pthen(pstring("fun"), pws()));
    let fun_binding = pright(pthen(fun_binding, pidentifier()));
    let fun_binding = pleft(pthen(fun_binding, pws()));
    let fun_binding = pleft(pthen(fun_binding, pchar('(')));
    let fun_binding = pleft(pthen(fun_binding, pws()));
    let fun_binding = pthen(fun_binding, psepby(pparam(), pthen(pchar(','), pws())));

    let fun_binding = pleft(pthen(fun_binding, pws()));
    let fun_binding = pleft(pthen(fun_binding, pchar(')')));
    let fun_binding = pleft(pthen(fun_binding, pws()));
    let fun_binding = pleft(pthen(fun_binding, pchar('{')));
    let fun_binding = pleft(pthen(fun_binding, pws()));

    let fun_binding = pmap(fun_binding, |(name, params)| Fun {
        name: name,
        params: params.value,
    });
    fun_binding
    //fun_binding = pthen(fun_binding, pmany(let_binding))
    //let fun_binding = pthen(fun_binding, pmany(let_binding));
    //let fun_binding = pleft(pthen(fun_binding, pws()));
    //let fun_binding = pleft(pthen(fun_binding, pchar('}')));
}
