#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::result_unit_err)]

use substrate_fixed::types::{I110F18, I32F32, I64F64, I96F32, U64F64};

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
impl_safe_div_for_fixed!(I96F32, I32F32, I64F64, I110F18, U64F64);

// /// Trait for safe conversion to primitive type P
// pub trait SafeToNum<T> {
//     /// Safe conversion to primitive type P
//     fn safe_to_num<P>(self) -> P
//     where
//         P: num_traits::Bounded + substrate_fixed::prelude::ToFixed + substrate_fixed::prelude::FromFixed;
// }

// impl<T> SafeToNum<T> for T
// where
//     T: substrate_fixed::traits::Fixed,
// {
//     fn safe_to_num<P>(self) -> P
//     where
//         P: num_traits::Bounded + substrate_fixed::prelude::ToFixed + substrate_fixed::prelude::FromFixed
//     {
//         match self.try_into() {
//             Ok(value) => value,
//             Err(_) => P::max_value(),
//         }
//     }
// }
