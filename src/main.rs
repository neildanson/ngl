mod parser_combinator;

use parser_combinator::*;

#[derive(Debug)]
enum Value {
    Number(i32),
    Bool(bool),
}

fn main() {
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
    let pnumber = pmap(pnumber, |n| Value::Number(n));

    let ptrue = pmap(pstring("true"), |_| true);
    let pfalse = pmap(pstring("false"), |_| false);
    let pbool = pmap(por(ptrue, pfalse), |b| Value::Bool(b));

    let pvalue = por(pnumber, pbool);

    let let_binding = pleft(pthen(pstring("let"), pws()));
    let let_binding = pright(pthen(let_binding, pidentifier()));
    let let_binding = pleft(pthen(let_binding, pws()));
    let let_binding = pleft(pthen(let_binding, pchar('=')));
    let let_binding = pleft(pthen(let_binding, pws()));
    let let_binding = pthen(let_binding, pvalue);
    let let_binding = pleft(pthen(let_binding, pws()));
    let let_binding = pleft(pthen(let_binding, pchar(';')));
    let let_binding = pleft(pthen(let_binding, pws()));

    let fun_binding = pleft(pthen(pstring("fun"), pws()));
    let fun_binding = pright(pthen(fun_binding, pidentifier()));
    let fun_binding = pleft(pthen(fun_binding, pws()));
    let fun_binding = pleft(pthen(fun_binding, pchar('(')));
    let fun_binding = pleft(pthen(fun_binding, pws()));
    let fun_binding = pthen(fun_binding, psepby(param_binding, pchar(',')));
    let fun_binding = pleft(pthen(fun_binding, pws()));
    let fun_binding = pleft(pthen(fun_binding, pchar(')')));
    let fun_binding = pleft(pthen(fun_binding, pws()));
    let fun_binding = pleft(pthen(fun_binding, pchar('{')));
    let fun_binding = pleft(pthen(fun_binding, pws()));
    let fun_binding = pthen(fun_binding, pmany(let_binding));
    let fun_binding = pleft(pthen(fun_binding, pws()));
    let fun_binding = pleft(pthen(fun_binding, pchar('}')));

    let result = fun_binding(
        "fun n(x: y) {
            let x = 1; 
            let y = 2;   
        }"
        .into(),
    );

    println!("{:?}", result);
}

fn pidentifier<'a>() -> impl Fn(ContinuationState<'a>) -> ParseResult<char> {
    pany(&[
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
        's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    ])
}

fn pws<'a>() -> impl Fn(ContinuationState<'a>) -> ParseResult<'a, Vec<Token<char>>> {
    pmany(pany(&[' ', '\n', '\t', '\r']))
}

fn param_binding<'a>() -> impl Fn(ContinuationState<'a>) -> ParseResult<(Token<char>, Token<char>)>
{
    let param_binding = pleft(pthen(pidentifier(), pws()));
    let param_binding = pleft(pthen(param_binding, pchar(':')));
    let param_binding = pleft(pthen(param_binding, pws()));
    let param_binding = pthen(param_binding, pidentifier());
    param_binding
}
