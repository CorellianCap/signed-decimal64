//! Rounding for `SignedDecimalU64<S>`.
//
// - `round_dp(dp, mode)`: round to `dp` fractional digits *while keeping the same scale S*.
// - `trunc()`, `floor()`, `ceil()` to an integer (i.e., `dp = 0`).
use decimal64::ScaleMetrics;

use crate::{from_unscaled, pow10_u64, SignedDecimalU64};

/// Rounding modes supported by this module.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RoundingMode {
    /// Toward zero (truncate).
    TowardZero,
    /// Away from zero if there's any discarded remainder.
    AwayFromZero,
    /// Toward +∞ (a.k.a. ceiling).
    Ceil,
    /// Toward -∞ (a.k.a. floor).
    Floor,
    /// To nearest; ties go away from zero.
    HalfUp,
    /// To nearest; ties go toward zero.
    HalfDown,
    /// To nearest; ties to even last-kept digit (bankers’ rounding).
    HalfEven,
}

impl<S: ScaleMetrics> SignedDecimalU64<S> {
    /// Truncate to an integer (dp = 0), toward zero.
    #[inline]
    pub fn trunc(self) -> Self {
        self.round_dp(0, RoundingMode::TowardZero)
    }

    /// Round down toward -∞ to an integer (dp = 0).
    #[inline]
    pub fn floor(self) -> Self {
        self.round_dp(0, RoundingMode::Floor)
    }

    /// Round up toward +∞ to an integer (dp = 0).
    #[inline]
    pub fn ceil(self) -> Self {
        self.round_dp(0, RoundingMode::Ceil)
    }

    /// Truncate to `dp` fractional digits (toward zero).
    #[inline]
    pub fn trunc_dp(self, dp: u32) -> Self {
        self.round_dp(dp, RoundingMode::TowardZero)
    }

    /// Checked version of `round_dp`: returns `None` on overflow.
    pub fn checked_round_dp(self, dp: u32, mode: RoundingMode) -> Option<Self> {
        let scale = S::SCALE as u32;
        let dp = dp.min(scale);
        let drop = scale - dp;
        if drop == 0 {
            return Some(self);
        }

        let unit = pow10_u64(drop);
        let u = self.unscaled();
        let q = u / unit;
        let r = u % unit;
        if r == 0 {
            return Some(Self::new(self.is_negative(), from_unscaled::<S>(q * unit)));
        }

        let inc = should_increment(q, r, unit, self.is_negative(), mode);
        let q2 = q.checked_add(inc as u64)?;
        let new_unscaled = q2.checked_mul(unit)?;
        Some(Self::new(
            self.is_negative(),
            from_unscaled::<S>(new_unscaled),
        ))
    }

    /// Round to `dp` fractional digits in **the same scale**.
    /// Panics on overflow to mirror the upstream "panic-on-overflow" operator semantics.
    #[inline]
    pub fn round_dp(self, dp: u32, mode: RoundingMode) -> Self {
        self.checked_round_dp(dp, mode)
            .expect("SignedDecimalU64::round_dp overflow")
    }

    /// Convert to another scale `T`, applying rounding if scaling down.
    /// Panics on overflow (use `checked_to_scale` for a fallible version).
    #[inline]
    pub fn to_scale<T: ScaleMetrics>(self, mode: RoundingMode) -> SignedDecimalU64<T> {
        self.checked_to_scale::<T>(mode)
            .expect("SignedDecimalU64::to_scale overflow")
    }

    /// Fallible conversion to another scale `T`, applying the given rounding `mode`
    /// when reducing precision. Returns `None` on overflow.
    pub fn checked_to_scale<T: ScaleMetrics>(
        self,
        mode: RoundingMode,
    ) -> Option<SignedDecimalU64<T>> {
        let s_from = S::SCALE as i32;
        let s_to = T::SCALE as i32;

        // Same scale: just reinterpret the unscaled integer.
        if s_from == s_to {
            return Some(SignedDecimalU64::<T>::new(
                self.is_negative(),
                from_unscaled::<T>(self.unscaled()),
            ));
        }

        let mag = self.unscaled();

        if s_to < s_from {
            // Scaling DOWN: divide by 10^(s_from - s_to) with rounding.
            let drop = (s_from - s_to) as u32;
            let unit = pow10_u64(drop);
            let q = mag / unit;
            let r = mag % unit;

            if r == 0 {
                return Some(SignedDecimalU64::<T>::new(
                    self.is_negative(),
                    from_unscaled::<T>(q),
                ));
            }

            let inc = should_increment(q, r, unit, self.is_negative(), mode);
            let q2 = q.checked_add(inc as u64)?;
            Some(SignedDecimalU64::<T>::new(
                self.is_negative(),
                from_unscaled::<T>(q2),
            ))
        } else {
            // Scaling UP: multiply by 10^(s_to - s_from); no rounding needed.
            let mul = pow10_u64((s_to - s_from) as u32);
            let new_unscaled = mag.checked_mul(mul)?;
            Some(SignedDecimalU64::<T>::new(
                self.is_negative(),
                from_unscaled::<T>(new_unscaled),
            ))
        }
    }
}

// ---------- helpers ----------

/// Decide whether to increment the kept digit, given quotient/remainder and mode.
#[inline]
fn should_increment(q: u64, r: u64, unit: u64, is_negative: bool, mode: RoundingMode) -> bool {
    if r == 0 {
        return false;
    }
    match mode {
        RoundingMode::TowardZero => false,
        RoundingMode::AwayFromZero => true,
        RoundingMode::Ceil => !is_negative, // positives round up, negatives truncate
        RoundingMode::Floor => is_negative, // negatives round "down" (more negative), positives truncate
        RoundingMode::HalfUp => (r << 1) >= unit,
        RoundingMode::HalfDown => (r << 1) > unit,
        RoundingMode::HalfEven => {
            let twice = r << 1;
            if twice > unit {
                true
            } else if twice < unit {
                false
            } else {
                // Exactly half: increment iff the last kept digit is odd
                (q & 1) == 1
            }
        }
    }
}
