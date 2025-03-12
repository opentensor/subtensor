#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::result_unit_err)]
#![cfg_attr(test, allow(clippy::arithmetic_side_effects))]
#![cfg_attr(test, allow(clippy::unwrap_used))]

use substrate_fixed::{
    traits::Fixed,
    types::{I32F32, I64F64, I96F32, I110F18, U64F64, U96F32, U110F18},
};

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
impl_safe_div_for_primitive!(u8, u16, u32, u64, i8, i16, i32, i64, usize);

/// Implementation of safe division trait for substrate fixed types
macro_rules! impl_safe_div_for_fixed {
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
impl_safe_div_for_fixed!(I96F32, I32F32, I64F64, I110F18, U110F18, U64F64, U96F32);

fn abs_diff<T: Fixed>(a: T, b: T) -> T {
    if a < b {
        b.saturating_sub(a)
    } else {
        a.saturating_sub(b)
    }
}

/// Safe sqrt with good precision
pub fn checked_sqrt<T: SafeDiv + Fixed>(value: T, epsilon: T) -> Option<T> {
    let zero: T = T::saturating_from_num(0);
    let two: T = T::saturating_from_num(2);

    if value < zero {
        return None;
    }

    let mut high: T = value;
    let mut low: T = zero;
    let mut middle: T = high.saturating_add(low).safe_div(two);

    let mut iteration: i32 = 0;
    let max_iterations = 128;
    let mut check_val: T = value.safe_div(middle);

    // Iterative approximation using bisection
    while abs_diff(check_val, middle) > epsilon {
        if check_val < middle {
            high = middle;
        } else {
            low = middle;
        }

        middle = high.saturating_add(low).safe_div(two);
        check_val = value.safe_div(middle);

        iteration = iteration.saturating_add(1);
        if iteration > max_iterations {
            break;
        }
    }

    Some(middle)
}

pub fn checked_pow_fixed<F: Fixed>(base: F, exponent: i32) -> Option<F> {
    if exponent == 0 {
        return Some(F::from_num(1));
    }

    if base == F::from_num(0) {
        if exponent < 0 {
            // Cannot raise zero to a negative power (division by zero)
            return None;
        }
        return Some(F::from_num(0)); // 0^(positive number) = 0
    }

    let mut result = F::from_num(1);
    let mut base = base;
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
        result = F::from_num(1).checked_div(result).unwrap_or_default();
    }

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use substrate_fixed::types::U110F18; // Assuming U110F18 is properly imported

    // Helper function for absolute difference
    fn abs_diff(a: U110F18, b: U110F18) -> U110F18 {
        if a > b { a - b } else { b - a }
    }

    #[test]
    fn test_checked_sqrt_positive_values() {
        let value: U110F18 = U110F18::from_num(4.0);
        let epsilon: U110F18 = U110F18::from_num(0.0001);

        let result: Option<U110F18> = checked_sqrt(value, epsilon);
        assert!(result.is_some());
        let sqrt_result: U110F18 = result.unwrap();
        let precise_sqrt: U110F18 = U110F18::from_num(4.0_f64.sqrt());
        assert!(abs_diff(sqrt_result, precise_sqrt) <= epsilon);
    }

    #[test]
    fn test_checked_sqrt_large_value() {
        let value: U110F18 = U110F18::from_num(1_000_000_000_000_000_000.0);
        let epsilon: U110F18 = U110F18::from_num(0.0001);

        let result: Option<U110F18> = checked_sqrt(value, epsilon);
        assert!(result.is_some());
        let sqrt_result: U110F18 = result.unwrap();
        let precise_sqrt: U110F18 = U110F18::from_num(1_000_000_000_000_000_000.0_f64.sqrt());
        assert!(abs_diff(sqrt_result, precise_sqrt) <= epsilon);
    }

    #[test]
    fn test_checked_sqrt_21m_tao_value() {
        let value: U110F18 = U110F18::from_num(441_000_000_000_000_000_000_000_000_000_000.0);
        let epsilon: U110F18 = U110F18::from_num(1000);

        let result: Option<U110F18> = checked_sqrt(value, epsilon);
        assert!(result.is_some());
        let sqrt_result: U110F18 = result.unwrap();
        let precise_sqrt: U110F18 =
            U110F18::from_num(441_000_000_000_000_000_000_000_000_000_000.0_f64.sqrt());
        assert!(abs_diff(sqrt_result, precise_sqrt) <= epsilon);
    }

    #[test]
    fn test_checked_sqrt_zero() {
        let value: U110F18 = U110F18::from_num(0.0);
        let epsilon: U110F18 = U110F18::from_num(0.0001);

        let result: Option<U110F18> = checked_sqrt(value, epsilon);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), U110F18::from_num(0.0));
    }

    #[test]
    fn test_checked_sqrt_precision() {
        let value: U110F18 = U110F18::from_num(2.0);
        let epsilon: U110F18 = U110F18::from_num(0.0001);

        let result: Option<U110F18> = checked_sqrt(value, epsilon);
        assert!(result.is_some());
        let sqrt_result: U110F18 = result.unwrap();
        let precise_sqrt: U110F18 = U110F18::from_num(2.0_f64.sqrt());
        assert!(abs_diff(sqrt_result, precise_sqrt) <= epsilon);
    }

    #[test]
    fn test_checked_sqrt_max_iterations() {
        let value: U110F18 = U110F18::from_num(2.0);
        let epsilon: U110F18 = U110F18::from_num(1e-30); // Very high precision
        let result: Option<U110F18> = checked_sqrt(value, epsilon);
        assert!(result.is_some()); // Check that it doesn't break, but may not be highly accurate
    }

    #[test]
    fn test_checked_pow_fixed() {
        let base = U64F64::from_num(2.5);
        let result = checked_pow_fixed(base, 3);
        assert_eq!(result, Some(U64F64::from_num(15.625)));

        let base = I32F32::from_num(1.5);
        let result = checked_pow_fixed(base, -2);

        assert!((result.unwrap() - I32F32::from_num(0.44444444)).abs() <= I32F32::from_num(0.0001));

        let result = checked_pow_fixed(I32F32::from_num(0), -1);
        assert_eq!(result, None);
    }
}
