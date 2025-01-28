#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::result_unit_err)]

use substrate_fixed::types::{I110F18, I32F32, I64F64, I96F32, U64F64, U110F18};

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
impl_safe_div_for_fixed!(I96F32, I32F32, I64F64, I110F18, U64F64, U110F18);

fn abs_diff(a: U110F18, b: U110F18) -> U110F18 {
    if a < b {
        b.saturating_sub(a)
    } else {
        a.saturating_sub(b)
    }
}

/// Safe sqrt with good precision
pub fn checked_sqrt(value: U110F18, epsilon: U110F18) -> Option<U110F18> {
    let zero: U110F18 = U110F18::from_num(0);
    let two: U110F18 = U110F18::from_num(2);

    if value < zero {
        return None;
    }

    let mut high: U110F18 = value;
    let mut low: U110F18 = zero;
    let mut middle: U110F18 = (high + low) / two;

    let mut iteration = 0;
    let max_iterations = 128;
    let mut check_val: U110F18 = value.safe_div(middle);

    // Iterative approximation using bisection
    while abs_diff(check_val, middle) > epsilon {
        if check_val < middle {
            high = middle;
        } else {
            low = middle;
        }

        middle = (high + low) / two;
        check_val = value.safe_div(middle);

        iteration += 1;
        if iteration > max_iterations {
            break;
        }
    }

    Some(middle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use substrate_fixed::types::U110F18; // Assuming U110F18 is properly imported

    // Helper function for absolute difference
    fn abs_diff(a: U110F18, b: U110F18) -> U110F18 {
        if a > b {
            a - b
        } else {
            b - a
        }
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
        let precise_sqrt: U110F18 = U110F18::from_num(441_000_000_000_000_000_000_000_000_000_000.0_f64.sqrt());
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
}
