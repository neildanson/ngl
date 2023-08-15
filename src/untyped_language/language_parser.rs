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
    let many_numbers = any_number.many1();
    let number_parser = pchar('-').optional().then(many_numbers);
    let pnumber = number_parser.map(move |(negate, value)| {
        let string: String = value.value.into_iter().map(|c| c.value).collect();
        let number = string.parse::<i32>().unwrap();
        match negate.value {
            Some(_) => -number,
            None => number,
        }
    });
    pnumber.map(Value::Number)
}

fn pbool<'a>() -> impl Parser<'a, Value> {
    let ptrue = pstring(TRUE).map(|_| true);
    let pfalse = pstring(FALSE).map(|_| false);
    ptrue.or(pfalse).map(Value::Bool)
}

pub fn pquoted_string<'a>() -> impl Parser<'a, Value> {
    let pquote = pchar('"');
    let pstring = pright(pquote.clone().then(pquote.take_until()));
    pstring.map(|string| Value::String(string.to_string()))
}

fn pvalue<'a>() -> impl Parser<'a, Value> {
    pchoice!(pint(), pbool(), pquoted_string())
}

//TODO disallow reserved words
pub fn pidentifier<'a>() -> impl Parser<'a, String> {
    let ident = pany(&ALPHA);
    let alpha_numeric = pany(&ALPHA_NUMERIC).many();
    let ident = ident.then(alpha_numeric);

    ident.map(|(start, rest)| {
        let mut result: String = rest.value.into_iter().map(|c| c.value).collect();
        result.insert(0, start.value);
        result
    })
}

pub fn pws<'a>() -> impl Parser<'a, Vec<Token<char>>> {
    pany(&WS).many()
}

fn pchar_ws<'a>(c: char) -> impl Parser<'a, char> {
    pleft(pchar(c).then(pws()))
}

fn pstring_ws(s: &str) -> impl Parser<&str> {
    pleft(pstring(s).then(pws()))
}

pub fn pterminator<'a>() -> impl Parser<'a, ()> {
    let psemi = pchar(';').map(|_| ());
    pleft(psemi.then(pws()))
}

pub fn pparam<'a>() -> impl Parser<'a, Parameter> {
    let param_binding = pleft(pidentifier().then(pws()));
    let param_binding = pleft(param_binding.then(pchar_ws(':')));
    let param_binding = param_binding.then(pidentifier());
    let param_binding = pleft(param_binding.then(pws()));
    param_binding.map(|(name, type_)| Parameter(name, type_))
}

pub fn pparams<'a>() -> impl Parser<'a, Vec<Token<Parameter>>> {
    let lparen = pchar_ws('(');
    let rparen = pchar_ws(')');
    let comma = pchar_ws(',');

    let param_list = pparam().sep_by(comma);

    param_list.between(lparen, rparen)
}

pub fn plet<'a>() -> impl Parser<'a, ExprOrStatement> {
    let let_binding = pstring_ws("let");
    let let_binding = pright(let_binding.then(pidentifier()));
    let let_binding = pleft(let_binding.then(pws()));
    let let_binding = pleft(let_binding.then(pchar_ws('=')));
    let let_binding = let_binding.then(pexpr());
    let let_binding = pleft(let_binding.then(pws()));
    let_binding.map(|(name, value)| ExprOrStatement::Statement(Statement::Let(name, value)))
}

pub fn pexpr<'a>() -> impl Parser<'a, Expr> {
    let value = pvalue().map(Expr::Value);
    pchoice!(value, pcall(), pidentifier().map(Expr::Ident))
}

pub fn pcall<'a>() -> impl Parser<'a, Expr> {
    let call_binding = pleft(pidentifier().then(pws()));
    let lparen = pchar_ws('(');
    let rparen = pchar_ws(')');

    let expr = pexpr();

    let params = expr.sep_by(pchar_ws(','));
    let params = params.between(lparen, rparen);

    let call_binding = call_binding.then(params);
    let call_binding = pleft(call_binding.then(pws()));
    call_binding.map(|(name, params)| Expr::Call(name, params.value))
}

pub fn pbody<'a>() -> impl Parser<'a, Vec<Token<ExprOrStatement>>> {
    let plbrace = pchar_ws('{');
    let prbrace = pchar_ws('}');

    let call = pcall().map(ExprOrStatement::Expr);
    let expr_or_statement = call.or(plet());
    let expr_or_statement = pleft(expr_or_statement.then(pterminator()));

    let pexprorstatement = expr_or_statement.many();
    pexprorstatement.between(plbrace, prbrace)
}

pub fn pfun<'a>() -> impl Parser<'a, Fun> {
    let fun_binding = pstring_ws(FUN);
    let fun_binding = pright(fun_binding.then(pidentifier()));
    let fun_binding = pleft(fun_binding.then(pws()));
    let fun_binding = fun_binding.then(pparams());
    let fun_binding = pleft(fun_binding.then(pws()));
    let fun_binding = pleft(fun_binding.then(pstring_ws("->")));
    let fun_binding = fun_binding.then(pidentifier());
    let fun_binding = pleft(fun_binding.then(pws()));
    let fun_binding = fun_binding.then(pbody());

    let fun_binding = fun_binding.map(|(name_and_params, body)| Fun {
        name: name_and_params.value.0.value.0,
        params: name_and_params.value.0.value.1.value,
        body: body.value,
        return_type: name_and_params.value.1,
    });
    fun_binding
}
