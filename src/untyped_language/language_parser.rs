use crate::parser_combinator::*;
use crate::pchoice;

use super::*;

const FUN: &str = "fun";
const _LET: &str = "let";
const _IF: &str = "if";
const _ELSE: &str = "else";
const TRUE: &str = "true";
const FALSE: &str = "false";
const _RESERVED: [&str; 6] = [FUN, _LET, _IF, _ELSE, TRUE, FALSE];

const NUMBERS: [char; 10] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9'];
const ALPHA: [char; 53] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l',
    'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '_',
];
const ALPHA_NUMERIC: [char; 63] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l',
    'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '_', '0', '1', '2', '3',
    '4', '5', '6', '7', '8', '9',
];
const WS: [char; 4] = [' ', '\n', '\t', '\r'];

pub(crate) fn pint<'a>() -> impl Parser<'a, Value> {
    let any_number = pany(&NUMBERS);
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
    pmap(pnumber, Value::Number)
}

fn pbool<'a>() -> impl Parser<'a, Value> {
    let ptrue = pmap(pstring(TRUE), |_| true);
    let pfalse = pmap(pstring(FALSE), |_| false);
    pmap(por(ptrue, pfalse), Value::Bool)
}

pub fn pquoted_string<'a>() -> impl Parser<'a, Value> {
    let pquote = pchar('"');
    let pstring = pright(pthen(pquote.clone(), ptake_until(pquote)));
    pmap(pstring, |string| Value::String(string.to_string()))
}

fn pvalue<'a>() -> impl Parser<'a, Value> {
    pchoice!(pint(), pbool(), pquoted_string())
}

//TODO disallow reserved words
pub fn pidentifier<'a>() -> impl Parser<'a, String> {
    let ident = pany(&ALPHA);
    let alpha_numeric = pmany(pany(&ALPHA_NUMERIC));
    let ident = pthen(ident, alpha_numeric);

    pmap(ident, |(start, rest)| {
        let mut result: String = rest.value.into_iter().map(|c| c.value).collect();
        result.insert(0, start.value);
        result
    })
}

pub fn pws<'a>() -> impl Parser<'a, Vec<Token<char>>> {
    pmany(pany(&WS))
}

pub fn pterminator<'a>() -> impl Parser<'a, ()> {
    let psemi = pmap(pchar(';'), |_| ());
    pleft(pthen(psemi, pws()))
}

pub fn pparam<'a>() -> impl Parser<'a, Parameter> {
    let param_binding = pleft(pthen(pidentifier(), pws()));
    let param_binding = pleft(pthen(param_binding, pchar(':')));
    let param_binding = pleft(pthen(param_binding, pws()));
    let param_binding = pthen(param_binding, pidentifier());
    let param_binding = pleft(pthen(param_binding, pws()));
    pmap(param_binding, |(name, type_)| Parameter(name, type_))
}

pub fn pparams<'a>() -> impl Parser<'a, Vec<Token<Parameter>>> {
    let lparen = pleft(pthen(pchar('('), pws()));
    let rparen = pleft(pthen(pchar(')'), pws()));
    let comma = pleft(pthen(pchar(','), pws()));

    let param_list = psepby(pparam(), comma);

    pbetween(lparen, param_list, rparen)
}

pub fn plet<'a>() -> impl Parser<'a, ExprOrStatement> {
    let let_binding = pleft(pthen(pstring("let"), pws()));
    let let_binding = pright(pthen(let_binding, pidentifier()));
    let let_binding = pleft(pthen(let_binding, pws()));
    let let_binding = pleft(pthen(let_binding, pchar('=')));
    let let_binding = pleft(pthen(let_binding, pws()));
    let let_binding = pthen(let_binding, pexpr());
    let let_binding = pleft(pthen(let_binding, pws()));
    pmap(let_binding, |(name, value)| {
        ExprOrStatement::Statement(Statement::Let(name, value))
    })
}

pub fn pexpr<'a>() -> impl Parser<'a, Expr> {
    let value = pmap(pvalue(), Expr::Value);
    pchoice!(value, pcall(), pmap(pidentifier(), Expr::Ident))
}

pub fn pcall<'a>() -> impl Parser<'a, Expr> {
    let call_binding = pleft(pthen(pidentifier(), pws()));
    let lparen = pleft(pthen(pchar('('), pws()));
    let rparen = pleft(pthen(pchar(')'), pws()));

    let expr = pexpr();

    let params = psepby(expr, pleft(pthen(pchar(','), pws())));
    let params = pbetween(lparen, params, rparen);

    let call_binding = pthen(call_binding, params);
    let call_binding = pleft(pthen(call_binding, pws()));
    pmap(call_binding, |(name, params)| {
        Expr::Call(name, params.value)
    })
}

pub fn pbody<'a>() -> impl Parser<'a, Vec<Token<ExprOrStatement>>> {
    let plbrace = pleft(pchar('{').then(pws()));
    let prbrace = pleft(pthen(pchar('}'), pws()));

    let call = pmap(pcall(), |call| ExprOrStatement::Expr(call));
    let expr_or_statement = por(call, plet());
    let expr_or_statement = pleft(pthen(expr_or_statement, pterminator()));

    let pexprorstatement = pmany(expr_or_statement);
    pbetween(plbrace, pexprorstatement, prbrace)
}

pub fn pfun<'a>() -> impl Parser<'a, Fun> {
    let fun_binding = pleft(pthen(pstring(FUN), pws()));
    let fun_binding = pright(pthen(fun_binding, pidentifier()));
    let fun_binding = pleft(pthen(fun_binding, pws()));
    let fun_binding = pthen(fun_binding, pparams());
    let fun_binding = pleft(pthen(fun_binding, pws()));
    let fun_binding = pleft(pthen(fun_binding, pstring("->")));
    let fun_binding = pleft(pthen(fun_binding, pws()));
    let fun_binding = pthen(fun_binding, pidentifier());
    let fun_binding = pleft(pthen(fun_binding, pws()));
    let fun_binding = pthen(fun_binding, pbody());

    let fun_binding = pmap(fun_binding, |(name_and_params, body)| Fun {
        name: name_and_params.value.0.value.0,
        params: name_and_params.value.0.value.1.value,
        body: body.value,
        return_type: name_and_params.value.1,
    });
    fun_binding
}
