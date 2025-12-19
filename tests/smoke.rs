use core::str::FromStr;
use decimal64::{U2, U4};
use signed_decimal64::SignedDecimalU64;

#[test]
fn smoke_add_roundtrip() {
    let a = SignedDecimalU64::<U2>::from_str("12.34").unwrap();
    let b = SignedDecimalU64::<U2>::from_str("-0.34").unwrap();
    assert_eq!((a.clone() + b.clone()).to_string(), "12.00");
    assert_eq!((a - b).to_string(), "12.68");
}

#[test]
fn macro_numeric_literal() {
    use signed_decimal64::sdec;
    let x = sdec!(U2, -12.34);
    assert!(x.is_negative());
    assert_eq!(x.to_string(), "-12.34");
}

#[test]
fn smoke_rounding() {
    use signed_decimal64::round::RoundingMode;
    let x = SignedDecimalU64::<U4>::from_str("-1.2350").unwrap();
    assert_eq!(
        x.clone().round_dp(2, RoundingMode::HalfEven).to_string(),
        "-1.2400"
    );
    assert_eq!(
        x.clone().round_dp(2, RoundingMode::HalfUp).to_string(),
        "-1.2400"
    );
    assert_eq!(x.clone().ceil().to_string(), "-1.0000");
    assert_eq!(x.clone().floor().to_string(), "-2.0000");
}

#[test]
fn unsigned_conversions() {
    use decimal64::{DecimalU64, U2};
    let x = SignedDecimalU64::<U2>::from(DecimalU64::<U2>::from_str("3.50").unwrap());
    let neg = -x;

    assert_eq!(x.into_unsigned().to_string(), "3.50");
    assert_eq!(x.try_into_unsigned().unwrap().to_string(), "3.50");
    assert!(neg.try_into_unsigned().is_none());

    let mag = neg.abs().into_unsigned();
    assert_eq!(mag.to_string(), "3.50");

    // optional: expect_non_negative panics on negative
    let res = std::panic::catch_unwind(|| {
        let _ = neg.expect_non_negative("should be >= 0");
    });
    assert!(res.is_err());
}
