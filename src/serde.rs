//! Serde support for `SignedDecimalU64<S>`.
//!
//! Human-readable: string (e.g., "-12.34").
//! Binary: signed unscaled i128 (e.g., -1234 for U2).
//!
//! Enable with crate feature `serde`.

// Note: this file is compiled as the `serde` module.
// Avoid name collisions with the external serde crate.
use ::serde as serde_crate;
#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
use alloc::string::String;

use core::{fmt, marker::PhantomData, str::FromStr};
use decimal64::{DecimalU64, ScaleMetrics};

use crate::serde::alloc::string::ToString;
use crate::{from_unscaled, SignedDecimalU64};

use self::serde_crate::{de, Deserialize, Deserializer, Serialize, Serializer};

// -------- Serialize --------

impl<S: ScaleMetrics> Serialize for SignedDecimalU64<S> {
    fn serialize<Ser>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error>
    where
        Ser: Serializer,
    {
        if serializer.is_human_readable() {
            // e.g. "-12.34" with the fixed scale's formatting.
            serializer.serialize_str(&self.to_string())
        } else {
            // Signed unscaled representation (binary-friendly).
            let v: i128 = if self.is_negative() {
                -(self.unscaled() as i128)
            } else {
                self.unscaled() as i128
            };
            serializer.serialize_i128(v)
        }
    }
}

// -------- Deserialize --------

impl<'de, S: ScaleMetrics> Deserialize<'de> for SignedDecimalU64<S> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor<S: ScaleMetrics>(PhantomData<S>);

        impl<'de, S: ScaleMetrics> de::Visitor<'de> for Visitor<S> {
            type Value = SignedDecimalU64<S>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "a decimal string or a signed unscaled integer")
            }

            // Human-readable inputs
            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                parse_hr::<S, E>(s)
            }
            fn visit_borrowed_str<E>(self, s: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_str(s)
            }
            fn visit_string<E>(self, s: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_str(&s)
            }

            // Binary (non human-readable) integer inputs
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_i128(v as i128)
            }
            fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                from_signed_unscaled::<S, E>(v)
            }
            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                from_signed_unscaled::<S, E>(v as i128)
            }
            fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                if v > u64::MAX as u128 {
                    return Err(E::custom("unscaled magnitude too large for u64"));
                }
                Ok(SignedDecimalU64::new(false, from_unscaled::<S>(v as u64)))
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_str(Visitor::<S>(PhantomData))
        } else {
            deserializer.deserialize_i128(Visitor::<S>(PhantomData))
        }
    }
}

// -------- Helpers --------

#[inline]
fn from_signed_unscaled<S: ScaleMetrics, E: de::Error>(v: i128) -> Result<SignedDecimalU64<S>, E> {
    // Reject the single value whose absolute value doesn't fit in i128.
    if v == i128::MIN {
        return Err(E::custom("signed unscaled i128 underflow"));
    }
    let neg = v < 0;
    let mag_u128 = if neg { (-v) as u128 } else { v as u128 };
    if mag_u128 > u64::MAX as u128 {
        return Err(E::custom("unscaled magnitude too large for u64"));
    }
    Ok(SignedDecimalU64::new(
        neg,
        from_unscaled::<S>(mag_u128 as u64),
    ))
}

fn parse_hr<S: ScaleMetrics, E: de::Error>(s_in: &str) -> Result<SignedDecimalU64<S>, E> {
    let s = s_in.trim();
    if s.is_empty() {
        return Err(E::custom("empty string"));
    }
    let (neg, digits) = match s.as_bytes()[0] {
        b'+' => (false, &s[1..]),
        b'-' => (true, &s[1..]),
        _ => (false, s),
    };
    let mag = DecimalU64::<S>::from_str(digits)
        .map_err(|_| E::custom("invalid decimal for this fixed scale"))?;
    Ok(SignedDecimalU64::new(neg, mag))
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::str::FromStr;
    use decimal64::U2;

    #[test]
    fn json_roundtrip() {
        let x = SignedDecimalU64::<U2>::from_str("-12.34").unwrap();
        let s = serde_json::to_string(&x).unwrap();
        assert_eq!(s, "\"-12.34\"");
        let y: SignedDecimalU64<U2> = serde_json::from_str(&s).unwrap();
        assert_eq!(y.to_string(), "-12.34");
    }
}
