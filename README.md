# signed-decimal64

A tiny signed wrapper around [`decimal64::DecimalU64<S>`](https://crates.io/crates/decimal64):
stores a sign bit plus a magnitude and guarantees there is no `-0`.

## Highlights

- `SignedDecimalU64<S>`: sign + `DecimalU64<S>` magnitude
- Operators and `checked_*` methods in `arithmetic`
- Rounding helpers and cross-scale conversion in `round`
- Optional Serde support (`--features serde`) serializing as strings for JSON
- Ergonomic macros: `sdec!` and `sdec_unscaled!`
- Criterion benches to exercise hot paths

## Example

```rust
use std::str::FromStr;
use decimal64::{DecimalU64, U2};
use signed_decimal64::SignedDecimalU64;

let a = SignedDecimalU64::<U2>::from_str("12.34").unwrap();
let b = SignedDecimalU64::<U2>::from_str("-0.34").unwrap();
assert_eq!((a + b).to_string(), "12.00");
assert_eq!((a - b).to_string(), "12.68");
```

## Benchmarks

```bash
cargo bench
# open target/criterion/report/index.html
```
