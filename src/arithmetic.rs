//! Arithmetic for `SignedDecimalU64<S>`.
//
// - `Add/Sub/Mul/Div` operators: panic on overflow/underflow/div-by-zero
//   (matching `DecimalU64<S>` operator semantics).
// - `checked_add/sub/mul/div`: return `Option<Self>` on failure.

use core::iter::Sum;
use core::mem;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};
use decimal64::{DecimalU64, ScaleMetrics};

use crate::SignedDecimalU64;

impl<S: ScaleMetrics> SignedDecimalU64<S> {
    /// Checked addition. Returns `None` on overflow.
    pub fn checked_add(self, rhs: Self) -> Option<Self> {
        let a_mag = self.mag;
        let b_mag = rhs.mag;
        let a_neg = self.negative && a_mag.unscaled != 0;
        let b_neg = rhs.negative && b_mag.unscaled != 0;

        match (a_neg, b_neg) {
            (false, false) => a_mag.checked_add(b_mag).map(|m| Self::new(false, m)),
            (true, true) => a_mag.checked_add(b_mag).map(|m| Self::new(true, m)),
            // opposite signs -> subtract smaller magnitude from larger; sign of the larger
            _ => {
                if a_mag.unscaled >= b_mag.unscaled {
                    a_mag.checked_sub(b_mag).map(|m| Self::new(a_neg, m))
                } else {
                    b_mag.checked_sub(a_mag).map(|m| Self::new(b_neg, m))
                }
            }
        }
    }

    /// Checked subtraction implemented via `checked_add(self, -rhs)`.
    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.checked_add(-rhs)
    }

    /// Checked multiplication. Returns `None` on overflow.
    pub fn checked_mul(self, rhs: Self) -> Option<Self> {
        let a_mag = self.mag;
        let b_mag = rhs.mag;

        if a_mag.unscaled == 0 || b_mag.unscaled == 0 {
            return Some(Self::ZERO);
        }

        let neg = (self.negative && a_mag.unscaled != 0) ^ (rhs.negative && b_mag.unscaled != 0);
        a_mag.checked_mul(b_mag).map(|m| Self::new(neg, m))
    }

    /// Checked division. Returns `None` on div-by-zero or overflow.
    pub fn checked_div(self, rhs: Self) -> Option<Self> {
        let a_mag = self.mag;
        let b_mag = rhs.mag;
        if b_mag.unscaled == 0 {
            return None;
        }
        if a_mag.unscaled == 0 {
            return Some(Self::ZERO);
        }
        let neg = (self.negative && a_mag.unscaled != 0) ^ (rhs.negative && b_mag.unscaled != 0);
        a_mag.checked_div(b_mag).map(|m| Self::new(neg, m))
    }
}

// --- Operator traits (panic on failure to match `DecimalU64` operators) ---

impl<S: ScaleMetrics> Add for SignedDecimalU64<S> {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        self.checked_add(rhs)
            .expect("SignedDecimalU64::add overflow")
    }
}

impl<S: ScaleMetrics> Sub for SignedDecimalU64<S> {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        self.checked_sub(rhs)
            .expect("SignedDecimalU64::sub overflow")
    }
}

impl<S: ScaleMetrics> Mul for SignedDecimalU64<S> {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        self.checked_mul(rhs)
            .expect("SignedDecimalU64::mul overflow")
    }
}

impl<S: ScaleMetrics> Div for SignedDecimalU64<S> {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        self.checked_div(rhs)
            .expect("SignedDecimalU64::div by zero or overflow")
    }
}

impl<S: ScaleMetrics> AddAssign for SignedDecimalU64<S> {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = core::mem::take(self) + rhs;
    }
}

impl<S: ScaleMetrics> SubAssign for SignedDecimalU64<S> {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = core::mem::take(self) - rhs;
    }
}

impl<S: ScaleMetrics> MulAssign for SignedDecimalU64<S> {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = core::mem::take(self) * rhs;
    }
}

impl<S: ScaleMetrics> DivAssign for SignedDecimalU64<S> {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        *self = core::mem::take(self) / rhs;
    }
}

// --- Iteration helpers ---

impl<S: ScaleMetrics> Sum for SignedDecimalU64<S> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(SignedDecimalU64::ZERO, |acc, x| acc + x)
    }
}

impl<'a, S: ScaleMetrics> Sum<&'a SignedDecimalU64<S>> for SignedDecimalU64<S> {
    fn sum<I: Iterator<Item = &'a SignedDecimalU64<S>>>(iter: I) -> Self {
        iter.fold(SignedDecimalU64::ZERO, |acc, x| {
            let v = SignedDecimalU64::<S>::new(
                x.is_negative(),
                crate::from_unscaled::<S>(x.unscaled()),
            );
            acc + v
        })
    }
}

impl<S: ScaleMetrics> Add<DecimalU64<S>> for SignedDecimalU64<S> {
    type Output = Self;
    #[inline]
    fn add(self, rhs: DecimalU64<S>) -> Self::Output {
        self + SignedDecimalU64::from(rhs)
    }
}

impl<S: ScaleMetrics> AddAssign<DecimalU64<S>> for SignedDecimalU64<S> {
    #[inline]
    fn add_assign(&mut self, rhs: DecimalU64<S>) {
        *self = mem::take(self) + SignedDecimalU64::from(rhs);
    }
}

impl<S: ScaleMetrics> Sub<DecimalU64<S>> for SignedDecimalU64<S> {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: DecimalU64<S>) -> Self::Output {
        self - SignedDecimalU64::from(rhs)
    }
}

impl<S: ScaleMetrics> SubAssign<DecimalU64<S>> for SignedDecimalU64<S> {
    #[inline]
    fn sub_assign(&mut self, rhs: DecimalU64<S>) {
        *self = mem::take(self) - SignedDecimalU64::from(rhs);
    }
}

// Add/Sub with &DecimalU64<S>
impl<S: ScaleMetrics> Add<&DecimalU64<S>> for SignedDecimalU64<S> {
    type Output = Self;
    #[inline]
    fn add(self, rhs: &DecimalU64<S>) -> Self::Output {
        let mag = DecimalU64::<S>::from_raw(rhs.unscaled);
        self + SignedDecimalU64::from_mag(mag)
    }
}

impl<S: ScaleMetrics> Sub<&DecimalU64<S>> for SignedDecimalU64<S> {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: &DecimalU64<S>) -> Self::Output {
        let mag = DecimalU64::<S>::from_raw(rhs.unscaled);
        self - SignedDecimalU64::from_mag(mag)
    }
}

// AddAssign/SubAssign with &DecimalU64<S> (no move from *self or *rhs)
impl<S: ScaleMetrics> AddAssign<&DecimalU64<S>> for SignedDecimalU64<S> {
    #[inline]
    fn add_assign(&mut self, rhs: &DecimalU64<S>) {
        let mag = DecimalU64::<S>::from_raw(rhs.unscaled);
        *self = mem::take(self) + SignedDecimalU64::from_mag(mag);
    }
}

impl<S: ScaleMetrics> SubAssign<&DecimalU64<S>> for SignedDecimalU64<S> {
    #[inline]
    fn sub_assign(&mut self, rhs: &DecimalU64<S>) {
        let mag = DecimalU64::<S>::from_raw(rhs.unscaled);
        *self = mem::take(self) - SignedDecimalU64::from_mag(mag);
    }
}

impl<S: ScaleMetrics> core::iter::Product for SignedDecimalU64<S> {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(SignedDecimalU64::ONE, |acc, x| acc * x)
    }
}

impl<'a, S: ScaleMetrics> core::iter::Product<&'a SignedDecimalU64<S>> for SignedDecimalU64<S> {
    fn product<I: Iterator<Item = &'a SignedDecimalU64<S>>>(iter: I) -> Self {
        iter.fold(SignedDecimalU64::ONE, |acc, x| {
            let v = SignedDecimalU64::<S>::new(
                x.is_negative(),
                crate::from_unscaled::<S>(x.unscaled()),
            );
            acc * v
        })
    }
}
