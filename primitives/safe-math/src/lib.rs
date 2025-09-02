#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::result_unit_err)]
#![cfg_attr(test, allow(clippy::arithmetic_side_effects))]
#![cfg_attr(test, allow(clippy::unwrap_used))]

use core::f64::consts::LN_2;

use sp_arithmetic::traits::UniqueSaturatedInto;
use substrate_fixed::traits::Fixed;

/// Safe division trait
pub trait SafeDiv {
    /// Safe division that returns supplied default value for division by zero
    fn safe_div_or(self, rhs: Self, def: Self) -> Self;
    /// Safe division that returns default value for division by zero
    fn safe_div(self, rhs: Self) -> Self;
}

/// Implementation of safe division trait for primitive types
macro_rules! impl_safe_div_for_primitive {
    ($($t:ty),*) => {
        $(
            impl SafeDiv for $t {
                fn safe_div_or(self, rhs: Self, def: Self) -> Self {
                    self.checked_div(rhs).unwrap_or(def)
                }

                fn safe_div(self, rhs: Self) -> Self {
                    self.checked_div(rhs).unwrap_or_default()
                }
            }
        )*
    };
}
impl_safe_div_for_primitive!(u8, u16, u32, u64, u128, i8, i16, i32, i64, usize);

pub trait FixedExt: Fixed {
    fn checked_pow<E>(&self, exponent: E) -> Option<Self>
    where
        E: UniqueSaturatedInto<i32>,
    {
        let exponent = exponent.unique_saturated_into();

        if exponent == 0 {
            return Some(Self::from_num(1));
        }

        if *self == Self::from_num(0) {
            if exponent < 0 {
                // Cannot raise zero to a negative power (division by zero)
                return None;
            }
            return Some(Self::from_num(0)); // 0^(positive number) = 0
        }

        let mut result = Self::from_num(1);
        let mut base = *self;
        let mut exp = exponent.unsigned_abs();

        // Binary exponentiation algorithm
        while exp > 0 {
            if exp & 1 != 0 {
                result = result.saturating_mul(base);
            }
            base = base.saturating_mul(base);
            exp >>= 1;
        }

        if exponent < 0 {
            result = Self::from_num(1).checked_div(result).unwrap_or_default();
        }

        Some(result)
    }

    /// Safe sqrt with good precision
    fn checked_sqrt(&self, epsilon: Self) -> Option<Self> {
        let zero = Self::saturating_from_num(0);
        let one = Self::saturating_from_num(1);
        let two = Self::saturating_from_num(2);

        if *self < zero {
            return None;
        }

        let mut high;
        let mut low;
        if *self > one {
            high = *self;
            low = zero;
        } else {
            high = one;
            low = *self;
        }

        let mut middle = high.saturating_add(low).safe_div(two);

        let mut iteration: i32 = 0;
        let max_iterations = 128;
        let mut check_val = self.safe_div(middle);

        // Iterative approximation using bisection
        while check_val.abs_diff(middle) > epsilon {
            if check_val < middle {
                high = middle;
            } else {
                low = middle;
            }

            middle = high.saturating_add(low).safe_div(two);
            check_val = self.safe_div(middle);

            iteration = iteration.saturating_add(1);
            if iteration > max_iterations {
                break;
            }
        }

        Some(middle)
    }

    /// Natural logarithm (base e)
    fn checked_ln(&self) -> Option<Self> {
        if *self <= Self::from_num(0) {
            return None;
        }

        // Constants
        let one = Self::from_num(1);
        let two = Self::from_num(2);
        let ln2 = Self::from_num(LN_2);

        // Find integer part of log2(x)
        let mut exp = 0i64;
        let mut y = *self;

        // Scale y to be between 1 and 2
        while y >= two {
            y = y.checked_div(two)?;
            exp = exp.checked_add(1)?;
        }
        while y < one {
            y = y.checked_mul(two)?;
            exp = exp.checked_sub(1)?;
        }

        // At this point, 1 <= y < 2
        let z = y.checked_sub(one)?;

        // For better accuracy, use more terms in the Taylor series
        let z2 = z.checked_mul(z)?;
        let z3 = z2.checked_mul(z)?;
        let z4 = z3.checked_mul(z)?;
        let z5 = z4.checked_mul(z)?;
        let z6 = z5.checked_mul(z)?;
        let z7 = z6.checked_mul(z)?;
        let z8 = z7.checked_mul(z)?;

        // More terms in the Taylor series for better accuracy
        // ln(1+z) = z - z²/2 + z³/3 - z⁴/4 + z⁵/5 - z⁶/6 + z⁷/7 - z⁸/8 + ...
        let ln_y = z
            .checked_sub(z2.checked_mul(Self::from_num(0.5))?)?
            .checked_add(z3.checked_mul(Self::from_num(1.0 / 3.0))?)?
            .checked_sub(z4.checked_mul(Self::from_num(0.25))?)?
            .checked_add(z5.checked_mul(Self::from_num(0.2))?)?
            .checked_sub(z6.checked_mul(Self::from_num(1.0 / 6.0))?)?
            .checked_add(z7.checked_mul(Self::from_num(1.0 / 7.0))?)?
            .checked_sub(z8.checked_mul(Self::from_num(0.125))?)?;

        // Final result: ln(x) = ln(y) + exp * ln(2)
        let exp_ln2 = Self::from_num(exp).checked_mul(ln2)?;
        ln_y.checked_add(exp_ln2)
    }

    /// Logarithm with arbitrary base
    fn checked_log(&self, base: Self) -> Option<Self> {
        // Check for invalid base
        if base <= Self::from_num(0) || base == Self::from_num(1) {
            return None;
        }

        // Calculate using change of base formula: log_b(x) = ln(x) / ln(b)
        let ln_x = self.checked_ln()?;
        let ln_base = base.checked_ln()?;

        ln_x.checked_div(ln_base)
    }

    /// Returns the largest integer less than or equal to the fixed-point number.
    fn checked_floor(&self) -> Option<Self> {
        // Approach using the integer and fractional parts
        if *self >= Self::from_num(0) {
            // For non-negative numbers, simply return the integer part
            return Some(Self::from_num(self.int()));
        }

        // For negative numbers
        let int_part = self.int();
        let frac_part = self.frac();

        if frac_part == Self::from_num(0) {
            // No fractional part, return as is
            return Some(*self);
        }

        // Has fractional part, we need to round down
        int_part.checked_sub(Self::from_num(1))
    }

    fn abs_diff(&self, b: Self) -> Self {
        if *self < b {
            b.saturating_sub(*self)
        } else {
            self.saturating_sub(b)
        }
    }

    fn safe_div_or(&self, rhs: Self, def: Self) -> Self {
        self.checked_div(rhs).unwrap_or(def)
    }

    fn safe_div(&self, rhs: Self) -> Self {
        self.checked_div(rhs).unwrap_or_default()
    }
}

impl<T: Fixed> FixedExt for T {}

#[cfg(test)]
mod tests {
    use core::f64::consts::LN_10;
    use substrate_fixed::types::*; // Assuming U110F18 is properly imported

    use super::*;

    #[test]
    fn test_checked_sqrt_positive_values() {
        let value: U110F18 = U110F18::from_num(4.0);
        let epsilon: U110F18 = U110F18::from_num(0.0001);

        let result: Option<U110F18> = value.checked_sqrt(epsilon);
        assert!(result.is_some());
        let sqrt_result: U110F18 = result.unwrap();
        let precise_sqrt: U110F18 = U110F18::from_num(4.0_f64.sqrt());
        assert!(sqrt_result.abs_diff(precise_sqrt) <= epsilon);
    }

    #[test]
    fn test_checked_sqrt_large_value() {
        let value: U110F18 = U110F18::from_num(1_000_000_000_000_000_000.0);
        let epsilon: U110F18 = U110F18::from_num(0.0001);

        let result = value.checked_sqrt(epsilon);
        assert!(result.is_some());
        let sqrt_result: U110F18 = result.unwrap();
        let precise_sqrt: U110F18 = U110F18::from_num(1_000_000_000_000_000_000.0_f64.sqrt());
        assert!(sqrt_result.abs_diff(precise_sqrt) <= epsilon);
    }

    #[test]
    fn test_checked_sqrt_21m_tao_value() {
        let value: U110F18 = U110F18::from_num(441_000_000_000_000_000_000_000_000_000_000.0);
        let epsilon: U110F18 = U110F18::from_num(1000);

        let result: Option<U110F18> = value.checked_sqrt(epsilon);
        assert!(result.is_some());
        let sqrt_result: U110F18 = result.unwrap();
        let precise_sqrt: U110F18 =
            U110F18::from_num(441_000_000_000_000_000_000_000_000_000_000.0_f64.sqrt());
        assert!(sqrt_result.abs_diff(precise_sqrt) <= epsilon);
    }

    #[test]
    fn test_checked_sqrt_zero() {
        let value: U110F18 = U110F18::from_num(0.0);
        let epsilon: U110F18 = U110F18::from_num(0.0001);

        let result: Option<U110F18> = value.checked_sqrt(epsilon);
        assert!(result.is_some());
        let sqrt_result: U110F18 = result.unwrap();
        assert!(sqrt_result.abs_diff(U110F18::from_num(0)) <= epsilon);
    }

    #[test]
    fn test_checked_sqrt_precision() {
        let value: U110F18 = U110F18::from_num(2.0);
        let epsilon: U110F18 = U110F18::from_num(0.0001);

        let result = value.checked_sqrt(epsilon);
        assert!(result.is_some());
        let sqrt_result: U110F18 = result.unwrap();
        let precise_sqrt: U110F18 = U110F18::from_num(2.0_f64.sqrt());
        assert!(sqrt_result.abs_diff(precise_sqrt) <= epsilon);
    }

    #[test]
    fn test_checked_sqrt_max_iterations() {
        let value: U110F18 = U110F18::from_num(2.0);
        let epsilon: U110F18 = U110F18::from_num(1e-30); // Very high precision
        let result = value.checked_sqrt(epsilon);
        assert!(result.is_some()); // Check that it doesn't break, but may not be highly accurate
    }

    #[test]
    fn test_checked_pow_fixed() {
        let result = U64F64::from_num(2.5).checked_pow(3u32);
        assert_eq!(result, Some(U64F64::from_num(15.625)));

        let result = I32F32::from_num(1.5).checked_pow(-2i64);

        assert!(
            (result.unwrap() - I32F32::from_num(0.44444444)).abs() <= I32F32::from_num(0.00001)
        );

        let result = I32F32::from_num(0).checked_pow(-1);
        assert!(result.is_none());
    }

    #[test]
    fn test_checked_ln() {
        // Natural logarithm
        assert!(
            I64F64::from_num(10.0)
                .checked_ln()
                .unwrap()
                .abs_diff(I64F64::from_num(LN_10))
                < I64F64::from_num(0.00001)
        );

        // Log of negative number should return None
        assert!(I64F64::from_num(-5.0).checked_ln().is_none());

        // Log of zero should return None
        assert!(I64F64::from_num(0.0).checked_ln().is_none());
    }

    #[test]
    fn test_checked_log() {
        let x = I64F64::from_num(10.0);

        // Log base 10
        assert!(
            x.checked_log(I64F64::from_num(10.0))
                .unwrap()
                .abs_diff(I64F64::from_num(1.0))
                < I64F64::from_num(0.00001)
        );

        // Log with invalid base should return None
        assert!(x.checked_log(I64F64::from_num(-2.0)).is_none());

        // Log with base 1 should return None
        assert!(x.checked_log(I64F64::from_num(1.0)).is_none());
    }

    #[test]
    fn test_checked_floor() {
        // Test cases: (input, expected floor result)
        let test_cases = [
            // Positive and negative integers (should remain unchanged)
            (0.0, 0.0),
            (1.0, 1.0),
            (5.0, 5.0),
            (-1.0, -1.0),
            (-5.0, -5.0),
            // Positive fractions (should truncate to integer part)
            (0.5, 0.0),
            (1.5, 1.0),
            (3.75, 3.0),
            (9.999, 9.0),
            // Negative fractions (should round down to next integer)
            (-0.1, -1.0),
            (-1.5, -2.0),
            (-3.75, -4.0),
            (-9.999, -10.0),
        ];

        for &(input, expected) in &test_cases {
            let x = I64F64::from_num(input);
            let expected = I64F64::from_num(expected);
            assert_eq!(x.checked_floor().unwrap(), expected,);
        }
    }
}
