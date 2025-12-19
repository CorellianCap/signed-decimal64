//! Literal and const-friendly constructors for `SignedDecimalU64<S>`.

#![forbid(unsafe_code)]

/// Create a `SignedDecimalU64<$scale>` from a string/number literal at **runtime**.
///
/// Parses using the underlying `DecimalU64<$scale>` parser and infers sign.
/// Panics on invalid input (use your `FromStr` impl for a fallible path).
///
/// ```rust
/// # use signed_decimal64::{sdec, SignedDecimalU64, U2};
/// let x = sdec!(U2, "-12.34");
/// assert!(x.is_negative());
/// assert_eq!(x.to_string(), "-12.34");
/// ```
#[macro_export]
macro_rules! sdec {
    ($scale:path, $lit:literal) => {{
        // Accept both string and numeric literals by stringifying the input and
        // trimming optional quotes from string literals.
        let raw = ::core::stringify!($lit);
        let s = raw
            .strip_prefix('"')
            .and_then(|s| s.strip_suffix('"'))
            .unwrap_or(raw);

        // Parse sign manually, then let DecimalU64<$scale> parse the magnitude.
        let (neg, digits) = if let Some(rest) = s.strip_prefix('-') {
            (true, rest)
        } else if let Some(rest) = s.strip_prefix('+') {
            (false, rest)
        } else {
            (false, s)
        };
        let mag = <$crate::DecimalU64<$scale> as ::core::str::FromStr>::from_str(digits)
            .expect("invalid decimal literal for this fixed scale");
        $crate::SignedDecimalU64::<$scale>::new(neg, mag)
    }};
}

/// Create a `SignedDecimalU64<$scale>` **in const contexts** from raw parts
/// (sign + unscaled magnitude). This avoids parsing and can be used in `const`.
///
/// ```rust
/// # use signed_decimal64::{sdec_unscaled, SignedDecimalU64, U4};
/// const FEE: SignedDecimalU64<U4> = sdec_unscaled!(U4, true, 25_000); // -2.5000
/// assert!(FEE.is_negative());
/// ```
///
/// ⚠️ This relies on `DecimalU64`’s `unscaled` field being available.
#[macro_export]
macro_rules! sdec_unscaled {
    ($scale:path, $negative:expr, $unscaled:expr) => {{
        $crate::SignedDecimalU64::<$scale>::new(
            $negative,
            $crate::DecimalU64::<$scale>::from_raw($unscaled),
        )
    }};
}
