use core::cmp::Ordering;
use core::str::FromStr;
use decimal64::{DecimalU64, U0, U1, U2, U3};
use signed_decimal64::{pow10_u64, round::RoundingMode, sdec, sdec_unscaled, SignedDecimalU64};

#[test]
fn pow10_and_sign_helpers() {
    assert_eq!(pow10_u64(0), 1);
    assert_eq!(pow10_u64(4), 10_000);
    assert_eq!(pow10_u64(19), 10_000_000_000_000_000_000);
    let neg_zero = SignedDecimalU64::<U2>::new(true, DecimalU64::<U2>::from_raw(0));
    assert!(!neg_zero.is_negative());
    assert_eq!(neg_zero.signum(), 0);
    let mut x = SignedDecimalU64::<U3>::from_str("-1.230").unwrap();
    assert!(x.is_negative());
    assert_eq!(x.into_unscaled_i128(), -1230);
    let y = x.abs();
    assert!(y.is_positive());
    x.abs_assign();
    assert!(x.is_positive());
}

#[test]
fn checked_arithmetic_and_overflow() {
    let x = SignedDecimalU64::<U2>::from_str("1.50").unwrap();
    let y = SignedDecimalU64::<U2>::from_str("-0.50").unwrap();
    assert_eq!(x.checked_add(y).unwrap().to_string(), "1.00");
    assert_eq!(x.checked_sub(y).unwrap().to_string(), "2.00");
    assert_eq!(x.checked_mul(y).unwrap().to_string(), "-0.75");
    assert_eq!(x.checked_div(y).unwrap().to_string(), "-3.00");
    let max = SignedDecimalU64::<U0>::new(false, DecimalU64::<U0>::from_raw(u64::MAX));
    assert!(max.checked_add(SignedDecimalU64::<U0>::ONE).is_none());
    assert!(max
        .checked_mul(SignedDecimalU64::<U0>::from_str("2").unwrap())
        .is_none());
    assert!(SignedDecimalU64::<U0>::ONE
        .checked_div(SignedDecimalU64::<U0>::ZERO)
        .is_none());
}

#[test]
fn iterator_sum_product() {
    let vals = [sdec!(U0, 1), sdec!(U0, -2), sdec!(U0, 3)];
    assert_eq!(
        vals.iter()
            .copied()
            .sum::<SignedDecimalU64<U0>>()
            .to_string(),
        "2"
    );
    assert_eq!(
        vals.iter()
            .copied()
            .product::<SignedDecimalU64<U0>>()
            .to_string(),
        "-6"
    );
}

#[test]
fn conversions_and_scaling() {
    let x = SignedDecimalU64::<U3>::try_from(-1234_i128).unwrap();
    assert_eq!(x.to_string(), "-1.234");
    let y: SignedDecimalU64<U1> = x.to_scale::<U1>(RoundingMode::HalfUp);
    assert_eq!(y.to_string(), "-1.2");
    let z: SignedDecimalU64<U3> = y.to_scale::<U3>(RoundingMode::TowardZero);
    assert_eq!(z.to_string(), "-1.200");
    let big = SignedDecimalU64::<U0>::new(false, DecimalU64::<U0>::from_raw(u64::MAX));
    assert!(big
        .checked_to_scale::<U1>(RoundingMode::TowardZero)
        .is_none());
}

#[test]
fn rounding_mode_variants() {
    use RoundingMode::*;
    let p = SignedDecimalU64::<U2>::from_str("1.25").unwrap();
    assert_eq!(p.round_dp(1, TowardZero).to_string(), "1.20");
    assert_eq!(p.round_dp(1, AwayFromZero).to_string(), "1.30");
    assert_eq!(p.round_dp(1, Ceil).to_string(), "1.30");
    assert_eq!(p.round_dp(1, Floor).to_string(), "1.20");
    assert_eq!(p.round_dp(1, HalfUp).to_string(), "1.30");
    assert_eq!(p.round_dp(1, HalfDown).to_string(), "1.20");
    assert_eq!(p.round_dp(1, HalfEven).to_string(), "1.20");
    let n = SignedDecimalU64::<U2>::from_str("-1.25").unwrap();
    assert_eq!(n.round_dp(1, TowardZero).to_string(), "-1.20");
    assert_eq!(n.round_dp(1, AwayFromZero).to_string(), "-1.30");
    assert_eq!(n.round_dp(1, Ceil).to_string(), "-1.20");
    assert_eq!(n.round_dp(1, Floor).to_string(), "-1.30");
    assert_eq!(n.round_dp(1, HalfUp).to_string(), "-1.30");
    assert_eq!(n.round_dp(1, HalfDown).to_string(), "-1.20");
    assert_eq!(n.round_dp(1, HalfEven).to_string(), "-1.20");
}

#[test]
fn ordering_and_equality() {
    let zero = SignedDecimalU64::<U0>::ZERO;
    let neg_zero = SignedDecimalU64::<U0>::new(true, DecimalU64::<U0>::from_raw(0));
    assert_eq!(zero, neg_zero);
    assert_eq!(zero.cmp(&neg_zero), Ordering::Equal);
    let a = SignedDecimalU64::<U0>::from_str("1").unwrap();
    let b = SignedDecimalU64::<U0>::from_str("-2").unwrap();
    assert!(b < zero);
    assert!(a > b);
}

const CONST_FEE: SignedDecimalU64<U2> = sdec_unscaled!(U2, true, 250);

#[test]
fn macro_unscaled_const() {
    assert!(CONST_FEE.is_negative());
    assert_eq!(CONST_FEE.to_string(), "-2.50");
}
