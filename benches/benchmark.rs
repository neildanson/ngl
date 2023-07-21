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

fn parse_int_success(c: &mut Criterion) {
    let any_number = pany(&['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']);
    let many_numbers = pmany(any_number);
    let number_parser = pthen(poptional(pchar('-')), many_numbers);

    let to_number = pmap(number_parser, move |(negate, value)| {
        let string: String = value.value.into_iter().map(|c| c.value).collect();
        let number = string.parse::<i32>().unwrap();
        match negate.value {
            Some(_) => -number,
            None => number,
        }
    });

    c.bench_function("Parse int Success", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let _ = black_box(to_number("-123456789".into()));
            }
        })
    });
}

criterion_group!(
    benches,
    parse_char_success,
    parse_string_success,
    parse_char_fail,
    parse_string_fail,
    parse_int_success
);
criterion_main!(benches);
