use core::str::FromStr;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use decimal64::{U2, U8};
use signed_decimal64::round::RoundingMode;
use signed_decimal64::{DecimalU64, SignedDecimalU64};
use std::hint::black_box;

fn mk_u8() -> Vec<SignedDecimalU64<U8>> {
    (0..1500u32)
        .map(|i| {
            let s = format!("{}.{:08}", i / 100, (i * 13) % 100_000_000);
            SignedDecimalU64::from(DecimalU64::<U8>::from_str(&s).unwrap())
        })
        .collect()
}

fn rescale_benches(c: &mut Criterion) {
    let v = mk_u8();
    let mut g = c.benchmark_group("rescale");
    g.throughput(Throughput::Elements(v.len() as u64));

    g.bench_with_input(
        BenchmarkId::new("down_u8_to_u2_half_even", "U8->U2"),
        &v,
        |b, data| {
            b.iter(|| {
                let mut acc = SignedDecimalU64::<U2>::ZERO;
                for x in data.iter() {
                    let x = SignedDecimalU64::<U8>::new(
                        x.is_negative(),
                        DecimalU64::<U8>::from_raw(x.unscaled()),
                    );
                    let vx = SignedDecimalU64::<U8>::new(
                        x.is_negative(),
                        DecimalU64::<U8>::from_raw(x.unscaled()),
                    );
                    acc = acc + black_box(vx).to_scale::<U2>(RoundingMode::HalfEven);
                }
                black_box(acc)
            })
        },
    );

    g.bench_with_input(BenchmarkId::new("up_u2_to_u8", "U2->U8"), &v, |b, _| {
        // Build U2 once (outside iter)
        let u2: Vec<_> = v
            .iter()
            .map(|x| {
                SignedDecimalU64::<U8>::new(
                    x.is_negative(),
                    DecimalU64::<U8>::from_raw(x.unscaled()),
                )
                .to_scale::<U2>(RoundingMode::TowardZero)
            })
            .collect();
        b.iter(|| {
            let mut acc = SignedDecimalU64::<U8>::ZERO;
            for x in u2.iter() {
                let x = SignedDecimalU64::<U2>::new(
                    x.is_negative(),
                    DecimalU64::<U2>::from_raw(x.unscaled()),
                );
                let vx = SignedDecimalU64::<U2>::new(
                    x.is_negative(),
                    DecimalU64::<U2>::from_raw(x.unscaled()),
                );
                acc = acc + black_box(vx).to_scale::<U8>(RoundingMode::TowardZero);
            }
            black_box(acc)
        })
    });

    g.finish();
}

criterion_group!(benches, rescale_benches);
criterion_main!(benches);
