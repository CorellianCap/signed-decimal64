use core::str::FromStr;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use decimal64::{ScaleMetrics, U2, U8};
use signed_decimal64::{DecimalU64, SignedDecimalU64};
use std::hint::black_box;

fn mk_pairs<S: ScaleMetrics>() -> (Vec<SignedDecimalU64<S>>, Vec<SignedDecimalU64<S>>) {
    let mut a = Vec::with_capacity(512);
    let mut b = Vec::with_capacity(512);
    let width = S::SCALE as usize;
    for i in 1..=512u32 {
        // Construct strings with exactly SCALE fractional digits, avoiding zeros
        // frac_a, frac_b are in [1 .. 10^scale - 1]
        let base = 10u128.pow(width as u32);
        let fa = ((i as u128 * 17) % (base - 1)) + 1;
        let fb = ((i as u128 * 29) % (base - 1)) + 1;
        let int_a = (i % 50) + 1; // [1..50] to avoid 0
        let int_b = ((i * 3) % 50) + 1;
        let sa = format!(
            "{int}.{frac:0width$}",
            int = int_a,
            frac = fa,
            width = width
        );
        let sb = format!(
            "{int}.{frac:0width$}",
            int = int_b,
            frac = fb,
            width = width
        );
        a.push(SignedDecimalU64::from(
            DecimalU64::<S>::from_str(&sa).unwrap(),
        ));
        b.push(SignedDecimalU64::from(
            DecimalU64::<S>::from_str(&sb).unwrap(),
        ));
    }
    (a, b)
}

fn bench_mul_div<S: ScaleMetrics>(c: &mut Criterion, label: &str) {
    let (a, b) = mk_pairs::<S>();
    let mut g = c.benchmark_group(format!("mul_div_{label}"));
    g.throughput(Throughput::Elements(a.len() as u64));

    g.bench_with_input(
        BenchmarkId::new("mul", label),
        &(a.as_slice(), b.as_slice()),
        |bch, (x, y)| {
            bch.iter(|| {
                // Multiply each pair separately to avoid overflow from repeated accumulation.
                let mut sink = SignedDecimalU64::<S>::ZERO;
                for i in 0..x.len() {
                    let xi = SignedDecimalU64::<S>::new(
                        x[i].is_negative(),
                        DecimalU64::<S>::from_raw(x[i].unscaled()),
                    );
                    let yi = SignedDecimalU64::<S>::new(
                        y[i].is_negative(),
                        DecimalU64::<S>::from_raw(y[i].unscaled()),
                    );
                    // Keep the product alive via a trivial accumulation with addition.
                    let prod = black_box(xi) * black_box(yi);
                    sink = sink + prod;
                }
                black_box(sink)
            })
        },
    );

    g.bench_with_input(
        BenchmarkId::new("div", label),
        &(a.as_slice(), b.as_slice()),
        |bch, (x, y)| {
            bch.iter(|| {
                // Divide each pair separately; denominators are guaranteed non-zero by construction.
                let mut sink = SignedDecimalU64::<S>::ZERO;
                for i in 0..x.len() {
                    let xi = SignedDecimalU64::<S>::new(
                        x[i].is_negative(),
                        DecimalU64::<S>::from_raw(x[i].unscaled()),
                    );
                    let yi = SignedDecimalU64::<S>::new(
                        y[i].is_negative(),
                        DecimalU64::<S>::from_raw(y[i].unscaled()),
                    );
                    let q = black_box(xi) / black_box(yi);
                    sink = sink + q;
                }
                black_box(sink)
            })
        },
    );

    g.finish();
}

fn mul_div_benches(c: &mut Criterion) {
    bench_mul_div::<U2>(c, "U2");
    bench_mul_div::<U8>(c, "U8");
}

criterion_group!(benches, mul_div_benches);
criterion_main!(benches);
