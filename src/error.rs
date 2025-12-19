//! Error types for `SignedDecimalU64<S>` and a `FromStr` impl.

#![forbid(unsafe_code)]

use core::{fmt, str::FromStr};
use decimal64::ScaleMetrics;

use crate::{DecimalU64, SignedDecimalU64};

/// Errors for arithmetic operations (used by fallible APIs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathError {
    /// Division by zero.
    DivisionByZero,
    /// Overflow/underflow in magnitude arithmetic.
    Overflow,
}

impl fmt::Display for MathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MathError::DivisionByZero => f.write_str("division by zero"),
            MathError::Overflow => f.write_str("overflow"),
        }
    }
}

/// Error returned when parsing a `SignedDecimalU64<S>` from a string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseSignedDecimalError {
    /// Empty or only a sign.
    Empty,
    /// The magnitude failed to parse for the fixed scale `S`.
    InvalidMagnitude,
}

impl fmt::Display for ParseSignedDecimalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseSignedDecimalError::Empty => f.write_str("empty string"),
            ParseSignedDecimalError::InvalidMagnitude => {
                f.write_str("invalid decimal literal for this fixed scale")
            }
        }
    }
}

pub type Result<T> = core::result::Result<T, MathError>;

impl<S: ScaleMetrics> FromStr for SignedDecimalU64<S> {
    type Err = ParseSignedDecimalError;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(ParseSignedDecimalError::Empty);
        }
        let (neg, rest) = match s.as_bytes()[0] {
            b'+' => (false, &s[1..]),
            b'-' => (true, &s[1..]),
            _ => (false, s),
        };
        if rest.is_empty() {
            return Err(ParseSignedDecimalError::Empty);
        }
        let mag = DecimalU64::<S>::from_str(rest)
            .map_err(|_| ParseSignedDecimalError::InvalidMagnitude)?;
        Ok(SignedDecimalU64::new(neg, mag))
    }
}
