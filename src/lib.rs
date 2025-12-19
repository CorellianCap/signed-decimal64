//! signed-decimal64: a signed wrapper over `decimal64::DecimalU64<S>`.
//!
//! This crate exposes `SignedDecimalU64<S>` which represents a sign bit plus
//! a `DecimalU64<S>` magnitude. It normalizes away negative zero.
//!
//! Modules:
//! - `arithmetic`: operators + checked_* helpers
//! - `round`: rounding utilities and cross-scale conversion
//! - `serde` (feature = "serde"): Serialize/Deserialize impls
//! - `macros`: `sdec!` and `sdec_unscaled!`
//! - `error`: parse & math error types
//!
//! The API mirrors the upstream decimal64 crate’s style: fixed scale via
//! `ScaleMetrics` (`U0..U8`), `FromStr` for parsing, `Display` for formatting.

#![forbid(unsafe_code)]
#![no_std]

use core::cmp::Ordering;
use core::fmt;

pub use decimal64::{DecimalU64, ScaleMetrics, U0, U1, U2, U3, U4, U5, U6, U7, U8};

/// Internal helper: build a DecimalU64<S> from an unscaled integer.
#[inline]
pub(crate) const fn from_unscaled<S: ScaleMetrics>(unscaled: u64) -> DecimalU64<S> {
    DecimalU64::<S>::from_raw(unscaled)
}

/// 10^n as `u64` for 0 <= n <= 19 (compile-time friendly).
#[inline]
pub const fn pow10_u64(n: u32) -> u64 {
    match n {
        0 => 1,
        1 => 10,
        2 => 100,
        3 => 1_000,
        4 => 10_000,
        5 => 100_000,
        6 => 1_000_000,
        7 => 10_000_000,
        8 => 100_000_000,
        9 => 1_000_000_000,
        10 => 10_000_000_000,
        11 => 100_000_000_000,
        12 => 1_000_000_000_000,
        13 => 10_000_000_000_000,
        14 => 100_000_000_000_000,
        15 => 1_000_000_000_000_000,
        16 => 10_000_000_000_000_000,
        17 => 100_000_000_000_000_000,
        18 => 1_000_000_000_000_000_000,
        19 => 10_000_000_000_000_000_000,
        _ => panic!("pow10_u64: exponent too large"),
    }
}

/// A signed fixed-scale decimal backed by `DecimalU64<S>` and an explicit sign.
///
/// Invariant: `negative == false` whenever `mag.unscaled == 0`.
#[derive(Debug, Copy, Clone)]
pub struct SignedDecimalU64<S: ScaleMetrics> {
    pub(crate) negative: bool,
    pub(crate) mag: DecimalU64<S>,
}

impl<S: ScaleMetrics> SignedDecimalU64<S> {
    /// Creates a new value from `negative` and `mag`, normalizing `-0` to `0`.
    pub const fn new(negative: bool, mag: DecimalU64<S>) -> Self {
        let neg = negative && mag.unscaled != 0;
        Self { negative: neg, mag }
    }

    /// Construct a positive value from a magnitude.
    pub const fn from_mag(mag: DecimalU64<S>) -> Self {
        Self::new(false, mag)
    }

    /// Consumes and returns `(negative, magnitude)` with normalized sign.
    pub const fn into_parts(self) -> (bool, DecimalU64<S>) {
        (self.negative && self.mag.unscaled != 0, self.mag)
    }

    /// Returns the raw unscaled magnitude (always non-negative).
    pub const fn unscaled(&self) -> u64 {
        self.mag.unscaled
    }

    /// Returns `true` if the value is strictly negative.
    pub const fn is_negative(&self) -> bool {
        self.negative && self.mag.unscaled != 0
    }

    /// Returns `true` if the value is zero.
    pub const fn is_zero(&self) -> bool {
        self.mag.unscaled == 0
    }

    /// Returns `true` if the value is strictly positive.
    pub const fn is_positive(&self) -> bool {
        !self.is_negative() && !self.is_zero()
    }

    /// Returns `-1`, `0`, or `1` depending on the sign.
    pub const fn signum(&self) -> i8 {
        if self.is_negative() {
            -1
        } else if self.is_zero() {
            0
        } else {
            1
        }
    }

    /// Returns a copy with the sign flipped (still no negative zero).
    pub const fn negated(self) -> Self {
        if self.mag.unscaled == 0 {
            self // stays non-negative zero
        } else {
            Self {
                negative: !self.negative,
                mag: self.mag,
            }
        }
    }

    /// Absolute value (keeps the same scale; no negative zero).
    #[inline]
    pub const fn abs(self) -> Self {
        // If mag is zero, `negative` is forced false; otherwise we just clear the sign.
        Self {
            negative: false,
            mag: self.mag,
        }
    }

    /// Make this value absolute in place.
    #[inline]
    pub fn abs_assign(&mut self) {
        // Sign normalization already ensures `negative` is false when
        // `mag.unscaled` is zero, so clearing the sign unconditionally is
        // sufficient.
        self.negative = false;
    }

    /// Zero constant.
    pub const ZERO: Self = Self {
        negative: false,
        mag: DecimalU64::<S>::ZERO,
    };

    /// One constant (value `1` in the fixed scale `S`).
    pub const ONE: Self = Self {
        negative: false,
        mag: DecimalU64::<S>::ONE,
    };

    /// Always return the underlying magnitude (drops sign if negative).
    /// No panic.
    #[inline]
    pub const fn into_unsigned(self) -> DecimalU64<S> {
        self.mag
    }

    /// Borrowing variant (doesn't move `self`).
    #[inline]
    pub const fn to_unsigned(&self) -> DecimalU64<S> {
        DecimalU64::<S>::from_raw(self.mag.unscaled)
    }

    /// Returns the signed unscaled integer representation as `i128`.
    /// Negative values are represented with a negative unscaled magnitude; zero is always non-negative.
    #[inline]
    pub const fn into_unscaled_i128(self) -> i128 {
        if self.negative && self.mag.unscaled != 0 {
            -(self.mag.unscaled as i128)
        } else {
            self.mag.unscaled as i128
        }
    }

    /// Fallible: only succeeds for non-negative values.
    /// Prefer this when you *expect* non-negative and want the type system to enforce it.
    #[inline]
    pub const fn try_into_unsigned(self) -> Option<DecimalU64<S>> {
        if self.is_negative() {
            None
        } else {
            Some(self.mag)
        }
    }

    /// Panicking helper for when business logic guarantees non-negative,
    /// and you want a loud failure if that invariant is broken.
    #[inline]
    pub fn expect_non_negative(self, msg: &str) -> DecimalU64<S> {
        assert!(!self.is_negative(), "{msg}");
        self.mag
    }
}

impl<S: ScaleMetrics> Default for SignedDecimalU64<S> {
    fn default() -> Self {
        Self::ZERO
    }
}

impl<S: ScaleMetrics> From<DecimalU64<S>> for SignedDecimalU64<S> {
    fn from(value: DecimalU64<S>) -> Self {
        Self::from_mag(value)
    }
}

impl<S: ScaleMetrics> From<SignedDecimalU64<S>> for (bool, DecimalU64<S>) {
    fn from(value: SignedDecimalU64<S>) -> Self {
        value.into_parts()
    }
}

// --- Formatting ---

impl<S: ScaleMetrics> fmt::Display for SignedDecimalU64<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_negative() {
            write!(f, "-{}", &self.mag)
        } else {
            write!(f, "{}", &self.mag)
        }
    }
}

// --- Eq/Hash with -0 == 0 ---

impl<S: ScaleMetrics> PartialEq for SignedDecimalU64<S> {
    fn eq(&self, other: &Self) -> bool {
        let na = self.is_negative();
        let nb = other.is_negative();
        na == nb && self.mag.unscaled == other.mag.unscaled
    }
}
impl<S: ScaleMetrics> Eq for SignedDecimalU64<S> {}

impl<S: ScaleMetrics> core::hash::Hash for SignedDecimalU64<S> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.is_negative().hash(state);
        self.mag.unscaled.hash(state);
    }
}

// --- Ordering: negatives < zero < positives; for negatives, larger magnitude means smaller value ---

impl<S: ScaleMetrics> Ord for SignedDecimalU64<S> {
    fn cmp(&self, other: &Self) -> Ordering {
        use core::cmp::Ordering::*;
        match (
            self.is_negative(),
            self.mag.unscaled == 0,
            other.is_negative(),
            other.mag.unscaled == 0,
        ) {
            (_, true, _, true) => Equal,
            (true, _, false, _) => Less,
            (false, _, true, _) => Greater,
            (false, _, false, _) => self.mag.unscaled.cmp(&other.mag.unscaled),
            (true, _, true, _) => other.mag.unscaled.cmp(&self.mag.unscaled),
        }
    }
}

impl<S: ScaleMetrics> PartialOrd for SignedDecimalU64<S> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// --- Unary negation ---

impl<S: ScaleMetrics> core::ops::Neg for SignedDecimalU64<S> {
    type Output = Self;
    fn neg(self) -> Self::Output {
        self.negated()
    }
}

// Public prelude for convenience.
pub mod prelude {
    pub use crate::{
        DecimalU64, ScaleMetrics, SignedDecimalU64, U0, U1, U2, U3, U4, U5, U6, U7, U8,
    };
}

// Submodules
pub mod arithmetic;
pub mod error;
pub mod macros;
pub mod round;

#[cfg(all(feature = "serde", feature = "alloc"))]
pub mod serde;

// Conversions from signed unscaled integers
impl<S: ScaleMetrics> core::convert::TryFrom<i128> for SignedDecimalU64<S> {
    type Error = crate::error::MathError;
    #[inline]
    fn try_from(value: i128) -> Result<Self, Self::Error> {
        let neg = value.is_negative();
        let abs = value
            .checked_abs()
            .ok_or(crate::error::MathError::Overflow)? as u128;
        if abs > (u64::MAX as u128) {
            return Err(crate::error::MathError::Overflow);
        }
        let mag = DecimalU64::<S>::from_raw(abs as u64);
        Ok(SignedDecimalU64::new(neg, mag))
    }
}

impl<S: ScaleMetrics> core::convert::TryFrom<i64> for SignedDecimalU64<S> {
    type Error = crate::error::MathError;
    #[inline]
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        SignedDecimalU64::<S>::try_from(value as i128)
    }
}
