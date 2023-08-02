use crate::parser_combinator::*;

use super::*;

const FUN: &str = "fun";
const LET: &str = "let";
const IF: &str = "if";
const ELSE: &str = "else";
const TRUE: &str = "true";
const FALSE: &str = "false";
const RESERVED: [&str; 6] = [FUN, LET, IF, ELSE, TRUE, FALSE];

pub(crate) fn pint<'a>() -> impl Parser<'a, Output = Value> {
    let any_number = pany(vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']);
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

fn pbool<'a>() -> impl Parser<'a, Output = Value> {
    let ptrue = pmap(pstring(TRUE), |_| true);
    let pfalse = pmap(pstring(FALSE), |_| false);
    pmap(por(ptrue, pfalse), Value::Bool)
}

fn pvalue<'a>() -> impl Parser<'a, Output = Value> {
    por(pint(), pbool())
}

//TODO disallow reserved words
pub fn pidentifier<'a>() -> impl Parser<'a, Output = Token<String>> {
    let alpha = || {
        let alpha_lower: Vec<_> = ('a'..='z').collect();
        let alpha_upper: Vec<_> = ('A'..='Z').collect();

        let mut alpha: Vec<char> = Vec::new();
        alpha.extend(alpha_upper);
        alpha.extend(alpha_lower);
        alpha.push('_');
        alpha
    };

    let ident = pany(alpha());

    let mut alpha_numeric = alpha();
    let numeric = ('0'..='9').collect::<Vec<_>>();
    alpha_numeric.extend(numeric);
    let alpha_numeric = pmany(pany(alpha_numeric));
    let ident = pthen(ident, alpha_numeric);

    pmap(ident, |(start, rest)| {
        let mut result: String = rest.value.into_iter().map(|c| c.value).collect();
        result.insert(0, start.value);
        Token::new(result, start.start, start.length + rest.length)
    })
}

pub fn pws<'a>() -> impl Parser<'a, Output = Vec<Token<char>>> {
    pmany(pany(vec![' ', '\n', '\t', '\r']))
}

pub fn pterminator<'a>() -> impl Parser<'a, Output = ()> {
    let psemi = pmap(pchar(';'), |_| ());
    pleft(pthen(psemi, pws()))
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

pub fn pparams<'a>() -> impl Parser<'a, Output = Vec<Token<Parameter>>> {
    let lparen = pleft(pthen(pchar('('), pws()));
    let rparen = pleft(pthen(pchar(')'), pws()));
    let comma = pleft(pthen(pchar(','), pws()));

    let param_list = psepby(pparam(), comma);

    pbetween(lparen, param_list, rparen)
}

pub fn plet<'a>() -> impl Parser<'a, Output = ExprOrStatement> {
    let let_binding = pleft(pthen(pstring("let"), pws()));
    let let_binding = pright(pthen(let_binding, pidentifier()));
    let let_binding = pleft(pthen(let_binding, pws()));
    let let_binding = pleft(pthen(let_binding, pchar('=')));
    let let_binding = pleft(pthen(let_binding, pws()));
    let let_binding = pthen(let_binding, pvalue());
    let let_binding = pleft(pthen(let_binding, pws()));
    pmap(let_binding, |(name, value)| {
        ExprOrStatement::Statement(Statement::Let(name.value, value))
    })
}

pub fn pcall<'a>() -> impl Parser<'a, Output = ExprOrStatement> {
    let call_binding = pleft(pthen(pidentifier(), pws()));
    let lparen = pleft(pthen(pchar('('), pws()));
    let rparen = pleft(pthen(pchar(')'), pws()));

    let ident_or_value = pmap(pidentifier(), |ident| Expr::Ident(ident));

    let params = psepby(ident_or_value, pchar(','));
    let params = pbetween(lparen, params, rparen);

    let call_binding = pthen(call_binding, params);
    pmap(call_binding, |(name, params)| {
        ExprOrStatement::Expr(Expr::Call(name.value, params.value))
    })
}

pub fn pbody<'a>() -> impl Parser<'a, Output = Vec<Token<ExprOrStatement>>> {
    let plbrace = pleft(pthen(pchar('{'), pws()));
    let prbrace = pleft(pthen(pchar('}'), pws()));

    let expr_or_statement = por(pcall(), plet());
    let expr_or_statement = pleft(pthen(expr_or_statement, pterminator()));

    let pexprorstatement = pmany(expr_or_statement);
    pbetween(plbrace, pexprorstatement, prbrace)
}

pub fn pfun<'a>() -> impl Parser<'a, Output = Fun> {
    let fun_binding = pleft(pthen(pstring(FUN), pws()));
    let fun_binding = pright(pthen(fun_binding, pidentifier()));
    let fun_binding = pleft(pthen(fun_binding, pws()));
    let fun_binding = pthen(fun_binding, pparams());
    let fun_binding = pleft(pthen(fun_binding, pws()));
    let fun_binding = pthen(fun_binding, pbody());

    let fun_binding = pmap(fun_binding, |(name_and_params, body)| Fun {
        name: name_and_params.value.0.value,
        params: name_and_params.value.1.value,
        body: body.value,
    });
    fun_binding
}
