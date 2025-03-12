#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::result_unit_err)]
#![cfg_attr(test, allow(clippy::arithmetic_side_effects))]
#![cfg_attr(test, allow(clippy::unwrap_used))]

use sp_arithmetic::traits::UniqueSaturatedInto;
use substrate_fixed::traits::Fixed;

/// Safe division trait
pub trait SafeDiv {
    /// Safe division that returns supplied default value for division by zero
    fn safe_div_or(self, rhs: Self, def: Self) -> Self;
    /// Safe division that returns default value for division by zero
    fn safe_div(self, rhs: Self) -> Self;
}

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
            base = self.saturating_mul(base);
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
        let two = Self::saturating_from_num(2);

        if *self < zero {
            return None;
        }

        let mut high = *self;
        let mut low = zero;
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
impl_safe_div_for_primitive!(u8, u16, u32, u64, i8, i16, i32, i64, usize);

#[cfg(test)]
mod tests {
    use super::*;
    use substrate_fixed::types::*; // Assuming U110F18 is properly imported

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
        assert_eq!(result.unwrap(), U110F18::from_num(0.0));
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

        assert!((result.unwrap() - I32F32::from_num(0.44444444)).abs() <= I32F32::from_num(0.0001));

        let result = I32F32::from_num(0).checked_pow(-1);
        assert_eq!(result, None);
    }
}
