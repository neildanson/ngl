extern crate ngl;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use ngl::parser_combinator::*;
use wson::parse;

const JSON: &str = r#"
{
    "name": "John Doe",
    "age": 43,
    "phones": [
        "+44 1234567",
        "+44 2345678"
    ]
}"#;

fn nom_json_parse(c: &mut Criterion) {
    c.bench_function("Parse JSON NOM", |b| {
        b.iter(|| {
            for _ in 0..50 {
                let _ = black_box(parse(JSON));
            }
        });
    });
}

criterion_group!(benches, nom_json_parse);
criterion_main!(benches);
