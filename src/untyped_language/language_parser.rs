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

pub(crate) fn pint<'a>() -> impl Parser<'a, Value> {
    let any_number = pany_range('0'..='9');
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
    let pstring = pquote.clone().then(pquote.take_until()).right();
    pstring.map(|string| Value::String(string.to_string()))
}

fn pvalue<'a>() -> impl Parser<'a, Value> {
    pchoice!(pint(), pbool(), pquoted_string())
}

fn prange<'a>() -> impl Parser<'a, Expr> {
    let pstart = pexpr().ws();
    let pend = pexpr().ws();
    let pmid = pstring("..").ws();
    pstart
        .then(pmid)
        .left()
        .then(pend)
        .map(|(start, end)| Expr::Range(Box::new(start), Box::new(end)))
}

//TODO disallow reserved words
pub fn pidentifier<'a>() -> impl Parser<'a, String> {
    let ident = pany_range('a'..='z')
        .or(pany_range('A'..='Z'))
        .or(pchar('_'));
    let alpha_numeric = pany_range('a'..='z')
        .or(pany_range('A'..='Z'))
        .or(pany_range('0'..='9'))
        .or(pchar('_'))
        .many();
    let ident = ident.then(alpha_numeric);

    ident.map(|(start, rest)| {
        let mut result: String = rest.value.into_iter().map(|c| c.value).collect();
        result.insert(0, start.value);
        result
    })
}

pub fn pterminator<'a>() -> impl Parser<'a, ()> {
    pchar(';').map(|_| ()).ws()
}

pub fn pparam<'a>() -> impl Parser<'a, Parameter> {
    let param_binding = pidentifier().ws();
    let param_binding = param_binding.then(pchar(':').ws()).left();
    let param_binding = param_binding.then(pidentifier()).ws();
    param_binding.map(|(name, type_)| Parameter(name, type_))
}

pub fn pparams<'a>() -> impl Parser<'a, Vec<Token<Parameter>>> {
    let lparen = pchar('(').ws();
    let rparen = pchar(')').ws();
    let comma = pchar(',').ws();

    let param_list = pparam().sep_by(comma);

    param_list.between(lparen, rparen)
}

pub fn plet<'a>() -> impl Parser<'a, Statement> {
    let let_binding = pstring("let").ws();
    let let_binding = let_binding.then(pidentifier()).right().ws();
    let let_binding = let_binding.then(pchar('=').ws()).left();
    let let_binding = let_binding.then(pexpr()).ws();
    let_binding.map(|(name, value)| Statement::Let(name, value))
}

pub fn pfor<'a>() -> impl Parser<'a, Statement> {
    let for_binding = pstring("for").ws();
    let for_binding = for_binding.then(pidentifier()).right().ws();
    let for_binding = for_binding.then(pchar('=').ws()).left();
    let for_binding = for_binding.then(prange()).ws();
    let for_binding = for_binding.then(pbody().ws());

    for_binding.map(|(name_and_expr, body)| {
        Statement::For(name_and_expr.value.0, name_and_expr.value.1, body.value)
    })
}

pub fn pif<'a>() -> impl Parser<'a, Expr> {
    let if_binding = pstring("if").ws();
    let if_binding = if_binding.then(pexpr()).right().ws();
    let if_binding = if_binding.then(pbody().ws());

    if_binding.map(|(expr, body)| Expr::If(Box::new(expr), body.value))
}

pub fn pcall<'a>() -> impl Parser<'a, Expr> {
    let call_binding = pidentifier().ws();
    let lparen = pchar('(').ws();
    let rparen = pchar(')').ws();

    let expr = pexpr();

    let params = expr.sep_by(pchar(',').ws());
    let params = params.between(lparen, rparen);

    call_binding
        .then(params)
        .ws()
        .map(|(name, params)| Expr::Call(name, params.value))
}

pub fn pexpr<'a>() -> impl Parser<'a, Expr> {
    let value = pvalue().map(Expr::Value);
    pchoice!(
        value,
        pif(),
        pcall(),
        pidentifier().map(Expr::Ident) //prange()
    )
    .ws()
}

pub fn pstatement<'a>() -> impl Parser<'a, Statement> {
    pchoice!(pfor(), plet()).ws()
}

pub fn pbody<'a>() -> impl Parser<'a, Vec<Token<ExprOrStatement>>> {
    let plbrace = pchar('{').ws();
    let prbrace = pchar('}').ws();

    let expr = pexpr().map(ExprOrStatement::Expr);
    let statement = pstatement().map(ExprOrStatement::Statement);
    let expr_or_statement = statement.or(expr);
    let expr_or_statement = expr_or_statement.then(pterminator()).left();

    let pexprorstatement = expr_or_statement.many1();
    pexprorstatement.between(plbrace, prbrace)
}

pub fn pfun<'a>() -> impl Parser<'a, Fun> {
    let fun_binding = pstring(FUN).ws();
    let fun_binding = fun_binding.then(pidentifier()).right().ws();
    let fun_binding = fun_binding.then(pparams()).ws();
    let fun_binding = fun_binding.then(pstring("->").ws()).left();
    let fun_binding = fun_binding.then(pidentifier()).ws();
    let fun_binding = fun_binding.then(pbody());

    let fun_binding = fun_binding.map(|(name_and_params, body)| Fun {
        name: name_and_params.value.0.value.0,
        params: name_and_params.value.0.value.1.value,
        body: body.value,
        return_type: name_and_params.value.1,
    });
    fun_binding
}
