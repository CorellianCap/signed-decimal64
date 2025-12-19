use core::str::FromStr;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use decimal64::{ScaleMetrics, U2, U8};
use signed_decimal64::{DecimalU64, SignedDecimalU64};
use std::hint::black_box;

fn mk_data<S: ScaleMetrics>() -> (Vec<SignedDecimalU64<S>>, Vec<SignedDecimalU64<S>>) {
    // 1024 deterministic values: 0.00..10.23 for U2 (or scaled equivalents)
    let mut pos = Vec::with_capacity(1024);
    for i in 0..1024u32 {
        let s = format!("{}.{:02}", i / 100, i % 100);
        let mag = DecimalU64::<S>::from_str(&s).unwrap();
        pos.push(SignedDecimalU64::from(mag));
    }
    let neg = pos
        .iter()
        .map(|x| SignedDecimalU64::<S>::new(true, DecimalU64::<S>::from_raw(x.unscaled())))
        .collect();
    (pos, neg)
}

fn bench_add_sub<S: ScaleMetrics>(c: &mut Criterion, label: &str) {
    let (pos, neg) = mk_data::<S>();
    let mut g = c.benchmark_group(format!("add_sub_{label}"));
    g.throughput(Throughput::Elements(pos.len() as u64));

    g.bench_with_input(BenchmarkId::new("add_same_sign", label), &pos, |b, data| {
        b.iter(|| {
            let mut acc = SignedDecimalU64::<S>::ZERO;
            for x in data.iter() {
                let x = SignedDecimalU64::<S>::new(
                    x.is_negative(),
                    DecimalU64::<S>::from_raw(x.unscaled()),
                );
                let v = SignedDecimalU64::<S>::new(
                    x.is_negative(),
                    DecimalU64::<S>::from_raw(x.unscaled()),
                );
                acc = black_box(acc) + black_box(v);
            }
            black_box(acc)
        })
    });

    g.bench_with_input(
        BenchmarkId::new("add_opposite_sign", label),
        &(pos.as_slice(), neg.as_slice()),
        |b, (p, n)| {
            b.iter(|| {
                let mut acc = SignedDecimalU64::<S>::ZERO;
                for i in 0..p.len() {
                    let px = SignedDecimalU64::<S>::new(
                        p[i].is_negative(),
                        DecimalU64::<S>::from_raw(p[i].unscaled()),
                    );
                    let nx = SignedDecimalU64::<S>::new(
                        n[i].is_negative(),
                        DecimalU64::<S>::from_raw(n[i].unscaled()),
                    );
                    acc = black_box(acc) + black_box(px) + black_box(nx);
                }
                black_box(acc)
            })
        },
    );

    g.bench_with_input(BenchmarkId::new("sub", label), &pos, |b, data| {
        b.iter(|| {
            let mut acc = SignedDecimalU64::<S>::ZERO;
            for x in data.iter() {
                let x = SignedDecimalU64::<S>::new(
                    x.is_negative(),
                    DecimalU64::<S>::from_raw(x.unscaled()),
                );
                let v = SignedDecimalU64::<S>::new(
                    x.is_negative(),
                    DecimalU64::<S>::from_raw(x.unscaled()),
                );
                acc = black_box(acc) - black_box(v);
            }
            black_box(acc)
        })
    });

    g.finish();
}

fn add_sub_benches(c: &mut Criterion) {
    bench_add_sub::<U2>(c, "U2");
    bench_add_sub::<U8>(c, "U8");
}

criterion_group!(benches, add_sub_benches);
criterion_main!(benches);
