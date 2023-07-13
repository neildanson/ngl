extern crate ngl;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use ngl::*;

fn parse_success(c: &mut Criterion) {
    let truthy_parser = por!(pchar!('t'), pchar!('f'));

    c.bench_function("Parse Success", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let _ = black_box(truthy_parser("t".into()));
            }
        })
    });
}

criterion_group!(benches, parse_success);
criterion_main!(benches);
