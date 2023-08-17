extern crate ngl;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use ngl::parser_combinator::*;
use ngl::pchoice;
use wson::parse;

const JSON: &str = r#"{
    "name": "John Doe",
    "age": 43,
    "phones": [
        "+44 1234567",
        "+44 2345678"
    ]
}"#;

#[derive(Clone)]
enum Value<'a> {
    Number(i32),
    String(&'a str),
    Array(Vec<Value<'a>>),
}

fn pint<'a>() -> impl Parser<'a, Value<'a>> {
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

fn pquoted_string_raw<'a>() -> impl Parser<'a, &'a str> {
    let pquote = pchar('"');
    pright(pquote.clone().then(pquote.take_until())).ws()
}

fn pquoted_string<'a>() -> impl Parser<'a, Value<'a>> {
    pquoted_string_raw().map(Value::String)
}

fn parray<'a>() -> impl Parser<'a, Value<'a>> {
    let comma = pchar(',').ws();

    let pvalue = pvalue();
    let pvalues = pvalue.sep_by(comma);
    pvalues
        .between(pchar('[').ws(), pchar(']').ws())
        .map(move |t| Value::Array(t.iter().map(|t| t.value.clone()).collect()))
}

fn pvalue<'a>() -> impl Parser<'a, Value<'a>> {
    pchoice!(pint(), pquoted_string(), parray())
}

fn ppair<'a>() -> impl Parser<'a, (&'a str, Value<'a>)> {
    let pcolon = pchar(':').ws();
    let pidentifier = pquoted_string_raw();
    let pvalue = pvalue();
    pidentifier
        .then(pcolon)
        .then(pvalue)
        .map(|(identifier, value)| (identifier.value.0.value, value.value))
}

fn json<'a>() -> impl Parser<'a, Vec<Token<(&'a str, Value<'a>)>>> {
    ppair()
        .sep_by(pchar(',').ws())
        .between(pchar('{').ws(), pchar('}').ws())
}

fn nom_json_parse(c: &mut Criterion) {
    c.bench_function("Parse JSON NOM", |b| {
        b.iter(|| {
            for _ in 0..50 {
                let _ = black_box(parse(JSON));
            }
        });
    });
}

fn ngl_json_parse(c: &mut Criterion) {
    let parser = json();
    c.bench_function("Parse JSON NGL", |b| {
        b.iter(|| {
            for _ in 0..50 {
                let _ = black_box(parser.parse(JSON.into()));
            }
        });
    });
}

criterion_group!(benches, nom_json_parse, ngl_json_parse);
criterion_main!(benches);
