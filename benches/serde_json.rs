#![cfg(feature = "serde")]
use core::str::FromStr;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use decimal64::U2;
use serde_json::{from_slice, to_vec};
use signed_decimal64::SignedDecimalU64;

fn serde_benches(c: &mut Criterion) {
    let x = SignedDecimalU64::<U2>::from_str("-1234.56").unwrap();

    c.bench_function("serde_json_serialize_string_human_readable", |b| {
        b.iter(|| black_box(to_vec(black_box(&x))).unwrap())
    });

    let bytes = to_vec(&x).unwrap();
    c.bench_function("serde_json_deserialize_string_human_readable", |b| {
        b.iter(|| {
            let y: SignedDecimalU64<U2> = black_box(from_slice(black_box(&bytes))).unwrap();
            black_box(y)
        })
    });
}

criterion_group!(benches, serde_benches);
criterion_main!(benches);
