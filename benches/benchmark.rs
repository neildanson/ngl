extern crate ngl;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use ngl::*;

fn parse_char_success(c: &mut Criterion) {
    let truthy_parser = por(pchar('t'), pchar('f'));

    c.bench_function("Parse Char Success", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let _ = black_box(truthy_parser("t".into()));
            }
        })
    });
}

fn parse_char_fail(c: &mut Criterion) {
    let truthy_parser = por(pchar('t'), pchar('f'));

    c.bench_function("Parse Char Fail", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let _ = black_box(truthy_parser("x".into()));
            }
        })
    });
}

fn parse_string_success(c: &mut Criterion) {
    let truthy_parser = por(pstring("true"), pstring("false"));
    c.bench_function("Parse String Success", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let _ = black_box(truthy_parser("true".into()));
            }
        })
    });
}

fn parse_string_fail(c: &mut Criterion) {
    let truthy_parser = por(pstring("true"), pstring("false"));

    c.bench_function("Parse String Fail", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let _ = black_box(truthy_parser("wrong".into()));
            }
        })
    });
}

criterion_group!(
    benches,
    parse_char_success,
    parse_string_success,
    parse_char_fail,
    parse_string_fail
);
criterion_main!(benches);
