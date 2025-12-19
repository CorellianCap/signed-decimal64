#![cfg(feature = "serde")]
use core::str::FromStr;
use decimal64::U2;
use signed_decimal64::SignedDecimalU64;

#[test]
fn binary_roundtrip() {
    let x = SignedDecimalU64::<U2>::from_str("-12.34").unwrap();
    let bytes = bincode::serialize(&x).unwrap();
    let y: SignedDecimalU64<U2> = bincode::deserialize(&bytes).unwrap();
    assert_eq!(x, y);
}
