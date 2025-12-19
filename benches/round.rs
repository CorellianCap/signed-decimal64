use core::str::FromStr;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use decimal64::{ScaleMetrics, U4, U8};
use signed_decimal64::round::RoundingMode;
use signed_decimal64::{DecimalU64, SignedDecimalU64};
use std::hint::black_box;

fn mk_vals<S: ScaleMetrics>() -> Vec<SignedDecimalU64<S>> {
    (0..2000u32)
        .map(|i| {
            let s = format!("{}.{:0width$}", i / 100, i % 100, width = S::SCALE as usize);
            SignedDecimalU64::from(DecimalU64::<S>::from_str(&s).unwrap())
        })
        .collect()
}

fn bench_round<S: ScaleMetrics>(c: &mut Criterion, label: &str) {
    let v = mk_vals::<S>();
    let mut g = c.benchmark_group(format!("round_{label}"));
    g.throughput(Throughput::Elements(v.len() as u64));

    for &(dp, ref mode, name) in &[
        (0, RoundingMode::TowardZero, "trunc0"),
        (0, RoundingMode::Floor, "floor0"),
        (0, RoundingMode::Ceil, "ceil0"),
        (2, RoundingMode::HalfEven, "dp2_half_even"),
        (2, RoundingMode::HalfUp, "dp2_half_up"),
    ] {
        g.bench_with_input(BenchmarkId::new(name, label), &v, |b, data| {
            b.iter(|| {
                let mut sum = SignedDecimalU64::<S>::ZERO;
                for x in data.iter() {
                    let x = SignedDecimalU64::<S>::new(
                        x.is_negative(),
                        DecimalU64::<S>::from_raw(x.unscaled()),
                    );
                    let vx = SignedDecimalU64::<S>::new(
                        x.is_negative(),
                        DecimalU64::<S>::from_raw(x.unscaled()),
                    );
                    let y = black_box(vx).round_dp(dp, *mode);
                    sum = sum + y;
                }
                black_box(sum)
            })
        });
    }
    g.finish();
}

fn round_benches(c: &mut Criterion) {
    bench_round::<U8>(c, "U8");
    bench_round::<U4>(c, "U4");
}

criterion_group!(benches, round_benches);
criterion_main!(benches);
