//! Deterministic decimal fixed-point arithmetic.
//!
//! The simulation never touches `f32`/`f64`: float results depend on the compiler, the target
//! and the instruction set (fused multiply-add, x87, SIMD width), and one ulp of difference
//! compounds into a different world after a few thousand ticks. [`Fixed`] is an `i64` count of
//! thousandths, so every operation is integer arithmetic and gives the same bits everywhere.

use core::fmt;
use core::iter::Sum;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};
use core::str::FromStr;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A public saturating operation: `const` and free of any bookkeeping, or, with the
/// `saturation-count` feature, one that records where it hit the bounds (`saturation`).
macro_rules! counted {
    ($(#[$doc:meta])* $name:ident, $raw:ident, $op:literal, ($($arg:ident: $ty:ty),*)) => {
        $(#[$doc])*
        #[cfg(not(feature = "saturation-count"))]
        #[inline]
        pub const fn $name(self, $($arg: $ty),*) -> Self {
            self.$raw($($arg),*)
        }
        $(#[$doc])*
        #[cfg(feature = "saturation-count")]
        #[inline]
        #[track_caller]
        pub fn $name(self, $($arg: $ty),*) -> Self {
            let r = self.$raw($($arg),*);
            saturation::check(r, $op);
            r
        }
    };
}

/// Where the arithmetic hit the bounds (feature `saturation-count`, for diagnosis): a value
/// that saturates silently is usually a sum or a product that outgrew the range, and the
/// result is wrong without any sign. Each public saturating operation and operator records its
/// caller's file and line when the result is `MAX` or `MIN`. Off, none of this is compiled.
#[cfg(feature = "saturation-count")]
pub mod saturation {
    use std::collections::BTreeMap;
    use std::panic::Location;
    use std::sync::Mutex;

    type Counts = BTreeMap<(&'static str, u32, &'static str), u64>;
    static COUNTS: Mutex<Counts> = Mutex::new(BTreeMap::new());

    #[inline]
    #[track_caller]
    pub(crate) fn check(v: super::Fixed, op: &'static str) {
        if v == super::Fixed::MAX || v == super::Fixed::MIN {
            record(Location::caller(), op);
        }
    }

    #[cold]
    fn record(at: &'static Location<'static>, op: &'static str) {
        let mut c = COUNTS.lock().unwrap_or_else(|e| e.into_inner());
        *c.entry((at.file(), at.line(), op)).or_insert(0) += 1;
    }

    /// Every place that saturated so far: (file, line, operation, times), most first.
    pub fn report() -> Vec<(&'static str, u32, &'static str, u64)> {
        let c = COUNTS.lock().unwrap_or_else(|e| e.into_inner());
        let mut v: Vec<_> = c.iter().map(|(&(f, l, o), &n)| (f, l, o, n)).collect();
        v.sort_by(|a, b| b.3.cmp(&a.3).then(a.cmp(b)));
        v
    }
}

/// Signed decimal fixed-point number: the value is `raw / 1000`, stored in an `i64`.
///
/// # Why a scale of 1000
///
/// The type has to hold world-economy money for a century and prices with sub-cent precision.
///
/// - **Range.** With 3 decimal digits the largest value is `i64::MAX / 1000 ≈ 9.22e15`. World
///   GDP in 2025 is about 1.1e14 dollars a year; the stated target is aggregates up to ~1e15
///   currency units, which leaves a ninefold margin. A scale of 10⁴ would top out at 9.2e14,
///   below the target.
/// - **Precision.** The resolution is 0.001, a tenth of a cent, so prices and money round to
///   below a cent. Because the scale is decimal, values written in data files (`0.05`, `19.99`)
///   are exact, unlike a binary scale.
/// - **Rates finer than 0.001** (a daily growth rate of 0.00003, say) are not stored as a
///   `Fixed`; they are applied as an exact integer ratio with [`Fixed::mul_ratio`], which goes
///   through `i128` and rounds once.
///
/// # Semantics
///
/// - The range is symmetric, `[-MAX, MAX]`, so negation and `abs` never overflow.
/// - Multiplication and division go through `i128` and round to nearest, ties away from zero.
/// - `checked_*` return `None` on overflow; `saturating_*` and the operators clamp to the range.
///   The `/` operator panics on a zero divisor, like integer division.
/// - `Sum` folds serially with saturation. Saturating addition is not associative, so parallel
///   reductions must collect in id order and fold serially, never `rayon::reduce`.
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Fixed(i64);

/// Rounds `n / d` to nearest, ties away from zero. `d` must not be zero.
#[inline]
const fn div_round(n: i128, d: i128) -> i128 {
    // Almost every product fits in 64 bits: divide there, in hardware, instead of in the
    // 128-bit software routine. Same integers, same rounding, same result. `i64::MIN` is left
    // out so that `MIN / -1` cannot overflow and `2 × |remainder|` fits in a `u64`.
    const LO: i128 = -(i64::MAX as i128);
    const HI: i128 = i64::MAX as i128;
    if n >= LO && n <= HI && d >= LO && d <= HI {
        let (n, d) = (n as i64, d as i64);
        let quotient = n / d;
        let remainder = n % d;
        return if remainder.unsigned_abs() * 2 >= d.unsigned_abs() {
            if (n < 0) != (d < 0) {
                quotient as i128 - 1
            } else {
                quotient as i128 + 1
            }
        } else {
            quotient as i128
        };
    }
    let quotient = n / d;
    let remainder = n % d;
    if remainder.unsigned_abs() * 2 >= d.unsigned_abs() {
        if (n < 0) != (d < 0) {
            quotient - 1
        } else {
            quotient + 1
        }
    } else {
        quotient
    }
}

/// Rounds `n / SCALE` to nearest, ties away from zero: [`div_round`] by the scale, in 64 bits,
/// where the constant divisor turns into a multiplication. `n` must not be `i64::MIN`.
#[inline]
const fn div_round_scale(n: i64) -> i64 {
    let quotient = n / Fixed::SCALE;
    let remainder = n % Fixed::SCALE;
    if remainder.unsigned_abs() * 2 >= Fixed::SCALE as u64 {
        if n < 0 { quotient - 1 } else { quotient + 1 }
    } else {
        quotient
    }
}

impl Fixed {
    /// Number of raw units in 1.
    pub const SCALE: i64 = 1_000;
    /// Number of decimal digits after the point.
    pub const FRAC_DIGITS: u32 = 3;
    /// Zero.
    pub const ZERO: Self = Self(0);
    /// One.
    pub const ONE: Self = Self(Self::SCALE);
    /// Smallest positive value, 0.001.
    pub const EPSILON: Self = Self(1);
    /// Largest value, about 9.22e15.
    pub const MAX: Self = Self(i64::MAX);
    /// Smallest value, `-MAX`.
    pub const MIN: Self = Self(-i64::MAX);

    /// From the raw count of thousandths. `i64::MIN` is clamped to [`Fixed::MIN`].
    #[inline]
    pub const fn from_raw(raw: i64) -> Self {
        if raw == i64::MIN {
            Self::MIN
        } else {
            Self(raw)
        }
    }

    /// The raw count of thousandths.
    #[inline]
    pub const fn raw(self) -> i64 {
        self.0
    }

    #[inline]
    const fn from_wide(value: i128) -> Option<Self> {
        if value > i64::MAX as i128 || value < -(i64::MAX as i128) {
            None
        } else {
            Some(Self(value as i64))
        }
    }

    #[inline]
    const fn saturate_wide(value: i128) -> Self {
        if value > i64::MAX as i128 {
            Self::MAX
        } else if value < -(i64::MAX as i128) {
            Self::MIN
        } else {
            Self(value as i64)
        }
    }

    /// From a whole number, `None` when it does not fit.
    #[inline]
    pub const fn checked_from_int(n: i64) -> Option<Self> {
        Self::from_wide(n as i128 * Self::SCALE as i128)
    }

    /// From a whole number, saturating.
    #[inline]
    pub const fn from_int(n: i64) -> Self {
        Self::saturate_wide(n as i128 * Self::SCALE as i128)
    }

    /// `num / den`, rounded, saturating.
    ///
    /// # Panics
    /// When `den` is zero.
    #[inline]
    pub const fn from_ratio(num: i64, den: i64) -> Self {
        assert!(den != 0, "Fixed::from_ratio with a zero denominator");
        Self::saturate_wide(div_round(num as i128 * Self::SCALE as i128, den as i128))
    }

    /// Integer part, rounded toward zero.
    #[inline]
    pub const fn to_int_trunc(self) -> i64 {
        self.0 / Self::SCALE
    }

    /// Integer part, rounded toward negative infinity.
    #[inline]
    pub const fn to_int_floor(self) -> i64 {
        self.0.div_euclid(Self::SCALE)
    }

    /// Nearest integer, ties away from zero.
    #[inline]
    pub const fn to_int_round(self) -> i64 {
        div_round(self.0 as i128, Self::SCALE as i128) as i64
    }

    /// `self + rhs`, `None` on overflow.
    #[inline]
    pub const fn checked_add(self, rhs: Self) -> Option<Self> {
        Self::from_wide(self.0 as i128 + rhs.0 as i128)
    }

    /// `self - rhs`, `None` on overflow.
    #[inline]
    pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
        Self::from_wide(self.0 as i128 - rhs.0 as i128)
    }

    /// `self * rhs`, rounded, `None` on overflow.
    #[inline]
    pub const fn checked_mul(self, rhs: Self) -> Option<Self> {
        Self::from_wide(div_round(
            self.0 as i128 * rhs.0 as i128,
            Self::SCALE as i128,
        ))
    }

    /// `self / rhs`, rounded, `None` on overflow or a zero divisor.
    #[inline]
    pub const fn checked_div(self, rhs: Self) -> Option<Self> {
        if rhs.0 == 0 {
            return None;
        }
        Self::from_wide(div_round(
            self.0 as i128 * Self::SCALE as i128,
            rhs.0 as i128,
        ))
    }

    /// `self + rhs`, saturating.
    #[inline]
    const fn saturating_add_raw(self, rhs: Self) -> Self {
        Self::saturate_wide(self.0 as i128 + rhs.0 as i128)
    }

    /// `self - rhs`, saturating.
    #[inline]
    const fn saturating_sub_raw(self, rhs: Self) -> Self {
        Self::saturate_wide(self.0 as i128 - rhs.0 as i128)
    }

    /// `self * rhs`, rounded, saturating.
    #[inline]
    const fn saturating_mul_raw(self, rhs: Self) -> Self {
        // Almost every product fits in 64 bits: multiply and round there. The same integers and
        // the same rounding as the wide path below (`i64::MIN` goes there too).
        if let Some(p) = self.0.checked_mul(rhs.0)
            && p != i64::MIN
        {
            return Self(div_round_scale(p));
        }
        Self::saturate_wide(div_round(
            self.0 as i128 * rhs.0 as i128,
            Self::SCALE as i128,
        ))
    }

    /// `self / rhs`, rounded, saturating. A zero divisor gives `MAX` or `MIN` by the sign of
    /// `self`, and zero for `0 / 0`.
    #[inline]
    const fn saturating_div_raw(self, rhs: Self) -> Self {
        if rhs.0 == 0 {
            return if self.0 > 0 {
                Self::MAX
            } else if self.0 < 0 {
                Self::MIN
            } else {
                Self::ZERO
            };
        }
        Self::saturate_wide(div_round(
            self.0 as i128 * Self::SCALE as i128,
            rhs.0 as i128,
        ))
    }

    /// `self * mul / div` with a single rounding, `None` on overflow or a zero divisor.
    #[inline]
    pub const fn checked_mul_div(self, mul: Self, div: Self) -> Option<Self> {
        if div.0 == 0 {
            return None;
        }
        Self::from_wide(div_round(self.0 as i128 * mul.0 as i128, div.0 as i128))
    }

    /// `self * mul / div` with a single rounding, saturating.
    ///
    /// # Panics
    /// When `div` is zero.
    #[inline]
    const fn mul_div_raw(self, mul: Self, div: Self) -> Self {
        assert!(div.0 != 0, "Fixed::mul_div with a zero divisor");
        Self::saturate_wide(div_round(self.0 as i128 * mul.0 as i128, div.0 as i128))
    }

    /// `self * num / den` for an exact integer ratio, `None` on overflow or `den == 0`.
    #[inline]
    pub const fn checked_mul_ratio(self, num: i64, den: i64) -> Option<Self> {
        if den == 0 {
            return None;
        }
        Self::from_wide(div_round(self.0 as i128 * num as i128, den as i128))
    }

    /// `self * num / den` for an exact integer ratio, one rounding, saturating. Use it for
    /// rates finer than the resolution: `pop.mul_ratio(3, 100_000)` is 0.003%.
    ///
    /// # Panics
    /// When `den` is zero.
    #[inline]
    const fn mul_ratio_raw(self, num: i64, den: i64) -> Self {
        assert!(den != 0, "Fixed::mul_ratio with a zero denominator");
        Self::saturate_wide(div_round(self.0 as i128 * num as i128, den as i128))
    }

    counted!(
        /// `self + rhs`, saturating.
        saturating_add, saturating_add_raw, "add", (rhs: Self)
    );
    counted!(
        /// `self - rhs`, saturating.
        saturating_sub, saturating_sub_raw, "sub", (rhs: Self)
    );
    counted!(
        /// `self * rhs`, rounded, saturating.
        saturating_mul, saturating_mul_raw, "mul", (rhs: Self)
    );
    counted!(
        /// `self / rhs`, rounded, saturating. A zero divisor gives `MAX` or `MIN` by the sign
        /// of `self`, and zero for `0 / 0`.
        saturating_div, saturating_div_raw, "div", (rhs: Self)
    );
    counted!(
        /// `self * mul / div` with a single rounding, saturating.
        ///
        /// # Panics
        /// When `div` is zero.
        mul_div, mul_div_raw, "mul_div", (mul: Self, div: Self)
    );
    counted!(
        /// `self * num / den` for an exact integer ratio, one rounding, saturating. Use it for
        /// rates finer than the resolution: `pop.mul_ratio(3, 100_000)` is 0.003%.
        ///
        /// # Panics
        /// When `den` is zero.
        mul_ratio, mul_ratio_raw, "mul_ratio", (num: i64, den: i64)
    );

    /// Absolute value (never overflows: the range is symmetric).
    #[inline]
    pub const fn abs(self) -> Self {
        Self(self.0.abs())
    }

    /// `-1`, `0` or `1` as a `Fixed`.
    #[inline]
    pub const fn signum(self) -> Self {
        Self(self.0.signum() * Self::SCALE)
    }

    /// Whether the value is below zero.
    #[inline]
    pub const fn is_negative(self) -> bool {
        self.0 < 0
    }

    /// Whether the value is above zero.
    #[inline]
    pub const fn is_positive(self) -> bool {
        self.0 > 0
    }

    /// **Display and conversion only, never in the simulation path.** Lossy `f64` view.
    pub fn to_f64_lossy(self) -> f64 {
        self.0 as f64 / Self::SCALE as f64
    }

    /// **Display and conversion only, never in the simulation path.** Rounds to the nearest
    /// thousandth and saturates; NaN becomes zero. Data files are parsed with [`str::parse`],
    /// which is exact; this exists for tools and charts.
    pub fn from_f64_lossy(value: f64) -> Self {
        if value.is_nan() {
            return Self::ZERO;
        }
        let scaled = (value * Self::SCALE as f64).round();
        if scaled >= i64::MAX as f64 {
            Self::MAX
        } else if scaled <= -(i64::MAX as f64) {
            Self::MIN
        } else {
            Self(scaled as i64)
        }
    }
}

impl Add for Fixed {
    type Output = Self;
    #[inline]
    #[cfg_attr(feature = "saturation-count", track_caller)]
    fn add(self, rhs: Self) -> Self {
        self.saturating_add(rhs)
    }
}

impl Sub for Fixed {
    type Output = Self;
    #[inline]
    #[cfg_attr(feature = "saturation-count", track_caller)]
    fn sub(self, rhs: Self) -> Self {
        self.saturating_sub(rhs)
    }
}

impl Mul for Fixed {
    type Output = Self;
    #[inline]
    #[cfg_attr(feature = "saturation-count", track_caller)]
    fn mul(self, rhs: Self) -> Self {
        self.saturating_mul(rhs)
    }
}

/// # Panics
/// On a zero divisor.
impl Div for Fixed {
    type Output = Self;
    #[inline]
    #[cfg_attr(feature = "saturation-count", track_caller)]
    fn div(self, rhs: Self) -> Self {
        assert!(rhs.0 != 0, "attempt to divide a Fixed by zero");
        self.saturating_div(rhs)
    }
}

/// Scales by a whole number, saturating.
impl Mul<i64> for Fixed {
    type Output = Self;
    #[inline]
    #[cfg_attr(feature = "saturation-count", track_caller)]
    fn mul(self, rhs: i64) -> Self {
        let r = Self::saturate_wide(self.0 as i128 * rhs as i128);
        #[cfg(feature = "saturation-count")]
        saturation::check(r, "mul_int");
        r
    }
}

/// Divides by a whole number, rounded.
///
/// # Panics
/// On a zero divisor.
impl Div<i64> for Fixed {
    type Output = Self;
    #[inline]
    #[cfg_attr(feature = "saturation-count", track_caller)]
    fn div(self, rhs: i64) -> Self {
        assert!(rhs != 0, "attempt to divide a Fixed by zero");
        Self::saturate_wide(div_round(self.0 as i128, rhs as i128))
    }
}

impl Neg for Fixed {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl AddAssign for Fixed {
    #[inline]
    #[cfg_attr(feature = "saturation-count", track_caller)]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl SubAssign for Fixed {
    #[inline]
    #[cfg_attr(feature = "saturation-count", track_caller)]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl MulAssign for Fixed {
    #[inline]
    #[cfg_attr(feature = "saturation-count", track_caller)]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl DivAssign for Fixed {
    #[inline]
    #[cfg_attr(feature = "saturation-count", track_caller)]
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl Sum for Fixed {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, |a, b| a + b)
    }
}

impl<'a> Sum<&'a Fixed> for Fixed {
    fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
        iter.copied().sum()
    }
}

/// Exact decimal. Without a precision, trailing zeros are dropped (`12.5`, `3`); with one,
/// the value is rounded (ties away from zero) to that many digits (`{:.2}` gives `12.50`).
impl fmt::Display for Fixed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let digits = Self::FRAC_DIGITS as usize;
        let mut raw = self.0 as i128;
        if let Some(precision) = f.precision()
            && precision < digits
        {
            let step = 10_i128.pow((digits - precision) as u32);
            raw = div_round(raw, step) * step;
        }
        let magnitude = raw.unsigned_abs();
        let int = magnitude / Self::SCALE as u128;
        let frac = format!("{:03}", magnitude % Self::SCALE as u128);
        let frac = match f.precision() {
            None => frac.trim_end_matches('0').to_string(),
            Some(p) if p <= digits => frac[..p].to_string(),
            Some(p) => format!("{frac:0<p$}"),
        };
        let digits = if frac.is_empty() {
            int.to_string()
        } else {
            format!("{int}.{frac}")
        };
        // `pad_integral` honours width, fill, `+` and `0` without truncating to the precision.
        f.pad_integral(raw >= 0, "", &digits)
    }
}

impl fmt::Debug for Fixed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fixed({self})")
    }
}

/// Why a string is not a [`Fixed`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseFixedError {
    /// No digits.
    Empty,
    /// A character other than a sign, digits and one point.
    InvalidDigit,
    /// More than three digits after the point (the value would not be exact).
    TooManyFractionDigits,
    /// Outside `[MIN, MAX]`.
    Overflow,
}

impl fmt::Display for ParseFixedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Empty => "empty fixed-point number",
            Self::InvalidDigit => "invalid character in fixed-point number",
            Self::TooManyFractionDigits => "more than 3 digits after the decimal point",
            Self::Overflow => "fixed-point number out of range",
        })
    }
}

impl std::error::Error for ParseFixedError {}

/// Exact parse of `[+-]digits[.digits]`, at most three digits after the point.
impl FromStr for Fixed {
    type Err = ParseFixedError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (negative, body) = match s.as_bytes().first() {
            Some(b'-') => (true, &s[1..]),
            Some(b'+') => (false, &s[1..]),
            _ => (false, s),
        };
        let (int, frac) = body.split_once('.').unwrap_or((body, ""));
        if int.is_empty() && frac.is_empty() {
            return Err(ParseFixedError::Empty);
        }
        if !int.bytes().chain(frac.bytes()).all(|b| b.is_ascii_digit()) {
            return Err(ParseFixedError::InvalidDigit);
        }
        if frac.len() > Self::FRAC_DIGITS as usize {
            return Err(ParseFixedError::TooManyFractionDigits);
        }
        let mut raw: i128 = 0;
        for b in int.bytes() {
            raw = raw * 10 + i128::from(b - b'0');
            if raw > i64::MAX as i128 {
                return Err(ParseFixedError::Overflow);
            }
        }
        raw *= Self::SCALE as i128;
        let mut frac_raw: i128 = 0;
        for b in frac.bytes() {
            frac_raw = frac_raw * 10 + i128::from(b - b'0');
        }
        raw += frac_raw * 10_i128.pow(Self::FRAC_DIGITS - frac.len() as u32);
        Self::from_wide(if negative { -raw } else { raw }).ok_or(ParseFixedError::Overflow)
    }
}

/// Human-readable formats (RON, JSON) get the exact decimal string; binary formats (postcard)
/// get the raw `i64`.
impl Serialize for Fixed {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if serializer.is_human_readable() {
            serializer.collect_str(self)
        } else {
            serializer.serialize_i64(self.0)
        }
    }
}

struct DecimalVisitor;

impl Visitor<'_> for DecimalVisitor {
    type Value = Fixed;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a decimal string with at most 3 fraction digits, or an integer")
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Fixed, E> {
        v.parse().map_err(E::custom)
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Fixed, E> {
        Fixed::checked_from_int(v).ok_or_else(|| E::custom(ParseFixedError::Overflow))
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Fixed, E> {
        i64::try_from(v)
            .ok()
            .and_then(Fixed::checked_from_int)
            .ok_or_else(|| E::custom(ParseFixedError::Overflow))
    }
}

struct RawVisitor;

impl Visitor<'_> for RawVisitor {
    type Value = Fixed;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a raw i64 fixed-point value")
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Fixed, E> {
        if v == i64::MIN {
            Err(E::custom(ParseFixedError::Overflow))
        } else {
            Ok(Fixed(v))
        }
    }
}

impl<'de> Deserialize<'de> for Fixed {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if deserializer.is_human_readable() {
            deserializer.deserialize_any(DecimalVisitor)
        } else {
            deserializer.deserialize_i64(RawVisitor)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Fixed, ParseFixedError, div_round, div_round_scale};
    use crate::rng::Rng;

    const MAX: i128 = i64::MAX as i128;

    fn reference(value: i128) -> Option<Fixed> {
        (-MAX..=MAX)
            .contains(&value)
            .then(|| Fixed::from_raw(value as i64))
    }

    fn clamp(value: i128) -> Fixed {
        Fixed::from_raw(value.clamp(-MAX, MAX) as i64)
    }

    /// Random raw values spread over every magnitude, with the edges mixed in.
    fn sample(rng: &mut Rng) -> i64 {
        match rng.below(16) {
            0 => 0,
            1 => i64::MAX,
            2 => -i64::MAX,
            3 => Fixed::SCALE * rng.range(-5, 6),
            _ => {
                let bits = rng.below(64) as u32;
                let magnitude = (rng.next_u64() >> 1) >> (63 - bits.min(63));
                let value = magnitude as i64;
                if rng.below(2) == 0 { value } else { -value }
            }
        }
    }

    #[test]
    fn arithmetic_matches_an_i128_model() {
        let mut rng = Rng::from_seed(7);
        for _ in 0..200_000 {
            let (a, b, c) = (sample(&mut rng), sample(&mut rng), sample(&mut rng));
            let (x, y, z) = (Fixed::from_raw(a), Fixed::from_raw(b), Fixed::from_raw(c));
            let (wa, wb, wc) = (a as i128, b as i128, c as i128);
            assert_eq!(x.checked_add(y), reference(wa + wb));
            assert_eq!(x.checked_sub(y), reference(wa - wb));
            assert_eq!(x.saturating_add(y), clamp(wa + wb));
            assert_eq!(x - y, clamp(wa - wb));
            let product = div_round(wa * wb, 1000);
            assert_eq!(x.checked_mul(y), reference(product));
            assert_eq!(x * y, clamp(product));
            if b != 0 {
                let quotient = div_round(wa * 1000, wb);
                assert_eq!(x.checked_div(y), reference(quotient));
                assert_eq!(x / y, clamp(quotient));
            }
            if c != 0 {
                assert_eq!(x.checked_mul_div(y, z), reference(div_round(wa * wb, wc)));
            }
            assert_eq!(x.checked_add(y), y.checked_add(x));
            assert_eq!(x.checked_mul(y), y.checked_mul(x));
            assert_eq!(-(-x), x);
            assert_eq!(x.abs().raw(), a.abs());
            assert_eq!(x < y, a < b);
        }
    }

    #[test]
    fn rounding_is_to_nearest_ties_away_from_zero() {
        assert_eq!(div_round(5, 2), 3);
        assert_eq!(div_round(-5, 2), -3);
        assert_eq!(div_round(5, -2), -3);
        assert_eq!(div_round(4, 3), 1);
        assert_eq!(div_round(-4, 3), -1);
        let third = Fixed::ONE / Fixed::from_int(3);
        assert_eq!(third.raw(), 333);
        assert_eq!((Fixed::from_int(2) / Fixed::from_int(3)).raw(), 667);
        assert_eq!(Fixed::from_raw(1500).to_int_round(), 2);
        assert_eq!(Fixed::from_raw(-1500).to_int_round(), -2);
        assert_eq!(Fixed::from_raw(-1500).to_int_floor(), -2);
        assert_eq!(Fixed::from_raw(-1500).to_int_trunc(), -1);
    }

    #[test]
    fn integer_ratios_keep_rates_below_the_resolution() {
        let population = Fixed::from_int(2_000_000_000);
        // 0.003% a day is below 0.001 but exact as a ratio.
        assert_eq!(population.mul_ratio(3, 100_000), Fixed::from_int(60_000));
        assert_eq!(Fixed::from_ratio(1, 3).raw(), 333);
        assert_eq!(Fixed::ONE.checked_mul_ratio(1, 0), None);
        assert_eq!(Fixed::MAX.checked_mul_ratio(2, 1), None);
        assert_eq!(Fixed::MAX.mul_ratio(2, 1), Fixed::MAX);
    }

    #[cfg(feature = "saturation-count")]
    #[test]
    fn saturation_is_counted_where_it_happens() {
        let big = Fixed::from_int(4_000_000_000_000);
        let line = line!() + 1;
        let product = big * big;
        assert_eq!(product, Fixed::MAX);
        let shares = big.mul_div(big, Fixed::from_int(1));
        assert_eq!(shares, Fixed::MAX);
        let report = super::saturation::report();
        assert!(
            report
                .iter()
                .any(|&(f, l, op, n)| f.ends_with("fixed.rs") && l == line && op == "mul" && n >= 1),
            "{report:?}"
        );
        assert!(report.iter().any(|&(_, l, op, _)| l == line + 2 && op == "mul_div"));
    }

    #[test]
    fn saturation_and_zero_division() {
        assert_eq!(Fixed::MAX + Fixed::ONE, Fixed::MAX);
        assert_eq!(Fixed::MIN - Fixed::ONE, Fixed::MIN);
        assert_eq!(Fixed::MAX * Fixed::from_int(-2), Fixed::MIN);
        assert_eq!(Fixed::ONE.checked_div(Fixed::ZERO), None);
        assert_eq!(Fixed::ONE.saturating_div(Fixed::ZERO), Fixed::MAX);
        assert_eq!((-Fixed::ONE).saturating_div(Fixed::ZERO), Fixed::MIN);
        assert_eq!(Fixed::ZERO.saturating_div(Fixed::ZERO), Fixed::ZERO);
        assert_eq!(Fixed::from_raw(i64::MIN), Fixed::MIN);
        assert_eq!(Fixed::checked_from_int(i64::MAX / 1000 + 1), None);
        assert_eq!(Fixed::from_int(i64::MAX), Fixed::MAX);
        let total: Fixed = [Fixed::MAX, Fixed::ONE, -Fixed::ONE].iter().sum();
        assert_eq!(total, Fixed::MAX - Fixed::ONE);
    }

    #[test]
    #[should_panic(expected = "divide a Fixed by zero")]
    fn division_operator_panics_on_zero() {
        let _ = Fixed::ONE / Fixed::ZERO;
    }

    #[test]
    fn display_and_parse_round_trip() {
        let mut rng = Rng::from_seed(11);
        for _ in 0..50_000 {
            let value = Fixed::from_raw(sample(&mut rng));
            assert_eq!(value.to_string().parse::<Fixed>(), Ok(value));
            assert_eq!(format!("{value:.3}").parse::<Fixed>(), Ok(value));
        }
        assert_eq!(Fixed::from_raw(12_500).to_string(), "12.5");
        assert_eq!(Fixed::from_int(3).to_string(), "3");
        assert_eq!(Fixed::from_raw(-5).to_string(), "-0.005");
        assert_eq!(format!("{:.2}", Fixed::from_raw(12_345)), "12.35");
        assert_eq!(format!("{:.0}", Fixed::from_raw(-400)), "0");
        assert_eq!(format!("{:.5}", Fixed::from_raw(1)), "0.00100");
        assert_eq!(format!("{:>8}", Fixed::from_raw(1500)), "     1.5");
        assert_eq!(format!("{:+08.2}", Fixed::from_raw(-1500)), "-0001.50");
        assert_eq!(Fixed::MAX.to_string(), "9223372036854775.807");
        assert_eq!("19.99".parse(), Ok(Fixed::from_raw(19_990)));
        assert_eq!("-.5".parse(), Ok(Fixed::from_raw(-500)));
        assert_eq!("+7.".parse(), Ok(Fixed::from_int(7)));
        assert_eq!("".parse::<Fixed>(), Err(ParseFixedError::Empty));
        assert_eq!("-".parse::<Fixed>(), Err(ParseFixedError::Empty));
        assert_eq!("1e3".parse::<Fixed>(), Err(ParseFixedError::InvalidDigit));
        assert_eq!("1.2.3".parse::<Fixed>(), Err(ParseFixedError::InvalidDigit));
        assert_eq!(
            "0.0001".parse::<Fixed>(),
            Err(ParseFixedError::TooManyFractionDigits)
        );
        assert_eq!(
            "9223372036854776".parse::<Fixed>(),
            Err(ParseFixedError::Overflow)
        );
    }

    #[test]
    fn serde_is_decimal_for_text_and_raw_for_binary() {
        let price = Fixed::from_raw(19_990);
        assert_eq!(ron::to_string(&price).unwrap(), "\"19.99\"");
        assert_eq!(ron::from_str::<Fixed>("\"19.99\"").unwrap(), price);
        assert_eq!(ron::from_str::<Fixed>("4").unwrap(), Fixed::from_int(4));
        assert!(
            ron::from_str::<Fixed>("4.5").is_err(),
            "floats are rejected"
        );
        let bytes = postcard::to_allocvec(&price).unwrap();
        assert_eq!(postcard::from_bytes::<Fixed>(&bytes).unwrap(), price);
    }

    #[test]
    fn float_helpers_are_only_approximate_views() {
        assert_eq!(Fixed::from_raw(1500).to_f64_lossy(), 1.5);
        assert_eq!(Fixed::from_f64_lossy(-2.25), Fixed::from_raw(-2250));
        assert_eq!(Fixed::from_f64_lossy(f64::NAN), Fixed::ZERO);
        assert_eq!(Fixed::from_f64_lossy(1e300), Fixed::MAX);
    }

    /// The 64-bit shortcut in `div_round` must give exactly what 128-bit division gives.
    #[test]
    fn div_round_matches_wide_division() {
        fn wide(n: i128, d: i128) -> i128 {
            let (q, r) = (n / d, n % d);
            if r.unsigned_abs() * 2 >= d.unsigned_abs() {
                if (n < 0) != (d < 0) { q - 1 } else { q + 1 }
            } else {
                q
            }
        }
        let edges = [
            0,
            1,
            -1,
            2,
            -2,
            999,
            1000,
            1001,
            -1000,
            500,
            -500,
            1500,
            -1500,
            MAX,
            -MAX,
            MAX - 1,
            -MAX + 1,
            MAX + 1,
            -MAX - 1,
            i64::MIN as i128,
            MAX * 1000,
            -MAX * 1000,
        ];
        for &n in &edges {
            for &d in &edges {
                if d != 0 {
                    assert_eq!(div_round(n, d), wide(n, d), "{n} / {d}");
                }
            }
        }
        let mut rng = Rng::from_seed(7);
        for _ in 0..100_000 {
            let n = i128::from(rng.next_u64() as i64) >> (rng.next_u64() % 64);
            let d = (i128::from(rng.next_u64() as i64) >> (rng.next_u64() % 64)).max(1);
            assert_eq!(div_round(n, d), wide(n, d), "{n} / {d}");
            assert_eq!(div_round(n, -d), wide(n, -d), "{n} / -{d}");
        }
    }

    /// The 64-bit product path of `*` against the 128-bit model: every pair of edge values
    /// (around zero, the rounding ties, the square root of the range where the product stops
    /// fitting in 64 bits, and the ends), then random pairs over every magnitude.
    #[test]
    fn multiplication_fast_path_matches_the_wide_model() {
        fn model(a: i64, b: i64) -> Fixed {
            clamp(div_round(a as i128 * b as i128, Fixed::SCALE as i128))
        }
        let root = 3_037_000_499i64; // floor(sqrt(i64::MAX))
        let mut edges = vec![i64::MAX, -i64::MAX, i64::MAX / 1000, -(i64::MAX / 1000)];
        for base in [0, 500, 1000, 1500, 2500, root, i64::MAX / 2, i64::MAX / 3] {
            for d in -3..=3 {
                if let Some(v) = base.checked_add(d) {
                    edges.push(v);
                    edges.push(-v);
                }
            }
        }
        edges.retain(|&v| v != i64::MIN);
        for &a in &edges {
            for &b in &edges {
                let got = Fixed::from_raw(a) * Fixed::from_raw(b);
                assert_eq!(got, model(a, b), "{a} * {b}");
            }
        }
        // Exhaustive over the rounding: every remainder of a product by the scale.
        for p in -5_000i64..=5_000 {
            assert_eq!(
                div_round_scale(p) as i128,
                div_round(p as i128, Fixed::SCALE as i128),
                "{p}"
            );
        }
        let mut rng = Rng::from_seed(11);
        for _ in 0..1_000_000 {
            let (a, b) = (sample(&mut rng), sample(&mut rng));
            assert_eq!(
                Fixed::from_raw(a) * Fixed::from_raw(b),
                model(a, b),
                "{a} * {b}"
            );
        }
    }
}
