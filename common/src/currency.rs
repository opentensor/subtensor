use core::fmt::{self, Display, Formatter};
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

#[cfg(feature = "approx")]
use approx::AbsDiffEq;
use codec::{
    Compact, CompactAs, Decode, DecodeWithMemTracking, Encode, Error as CodecError, MaxEncodedLen,
};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use substrate_fixed::traits::{Fixed, ToFixed};
use subtensor_macros::freeze_struct;

#[freeze_struct("40205476b6d995b2")]
#[repr(transparent)]
#[derive(
    Deserialize,
    Serialize,
    Clone,
    Copy,
    Decode,
    DecodeWithMemTracking,
    Default,
    Encode,
    Eq,
    Hash,
    MaxEncodedLen,
    Ord,
    PartialEq,
    PartialOrd,
    RuntimeDebug,
)]
pub struct AlphaCurrency(u64);

#[freeze_struct("4d1bcb31c40c2594")]
#[repr(transparent)]
#[derive(
    Deserialize,
    Serialize,
    Clone,
    Copy,
    Decode,
    DecodeWithMemTracking,
    Default,
    Encode,
    Eq,
    Hash,
    MaxEncodedLen,
    Ord,
    PartialEq,
    PartialOrd,
    RuntimeDebug,
)]
pub struct TaoCurrency(u64);

// implements traits required by the Currency trait (ToFixed + Into<u64> + From<u64>) and CompactAs,
// TypeInfo and Display. It expects a wrapper structure for u64 (CurrencyT(u64)).
macro_rules! impl_currency_reqs {
    ($currency_type:ident) => {
        impl $currency_type {
            pub const fn new(inner: u64) -> Self {
                Self(inner)
            }
        }

        impl TypeInfo for $currency_type {
            type Identity = <u64 as TypeInfo>::Identity;
            fn type_info() -> scale_info::Type {
                <u64 as TypeInfo>::type_info()
            }
        }

        impl Display for $currency_type {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                Display::fmt(&self.0, f)
            }
        }

        impl CompactAs for $currency_type {
            type As = u64;

            fn encode_as(&self) -> &Self::As {
                &self.0
            }

            fn decode_from(v: Self::As) -> Result<Self, CodecError> {
                Ok(Self(v))
            }
        }

        impl From<Compact<$currency_type>> for $currency_type {
            fn from(c: Compact<$currency_type>) -> Self {
                c.0
            }
        }

        impl From<$currency_type> for u64 {
            fn from(val: $currency_type) -> Self {
                val.0
            }
        }

        impl From<u64> for $currency_type {
            fn from(value: u64) -> Self {
                Self(value)
            }
        }

        impl ToFixed for $currency_type {
            fn to_fixed<F: Fixed>(self) -> F {
                self.0.to_fixed()
            }

            fn checked_to_fixed<F: Fixed>(self) -> Option<F> {
                self.0.checked_to_fixed()
            }

            fn saturating_to_fixed<F: Fixed>(self) -> F {
                self.0.saturating_to_fixed()
            }
            fn wrapping_to_fixed<F: Fixed>(self) -> F {
                self.0.wrapping_to_fixed()
            }

            fn overflowing_to_fixed<F: Fixed>(self) -> (F, bool) {
                self.0.overflowing_to_fixed()
            }
        }
    };
}

macro_rules! impl_arithmetic_operators {
    ($currency_type:ident) => {
        impl Add for $currency_type {
            type Output = Self;

            #[allow(clippy::arithmetic_side_effects)]
            fn add(self, rhs: Self) -> Self::Output {
                let lhs_u64: u64 = self.into();
                let rhs_u64: u64 = rhs.into();
                (lhs_u64 + rhs_u64).into()
            }
        }

        impl Sub for $currency_type {
            type Output = Self;

            #[allow(clippy::arithmetic_side_effects)]
            fn sub(self, rhs: Self) -> Self::Output {
                let lhs_u64: u64 = self.into();
                let rhs_u64: u64 = rhs.into();
                (lhs_u64 - rhs_u64).into()
            }
        }

        impl Mul for $currency_type {
            type Output = Self;

            #[allow(clippy::arithmetic_side_effects)]
            fn mul(self, rhs: Self) -> Self::Output {
                let lhs_u64: u64 = self.into();
                let rhs_u64: u64 = rhs.into();
                (lhs_u64 * rhs_u64).into()
            }
        }

        impl Div for $currency_type {
            type Output = Self;

            #[allow(clippy::arithmetic_side_effects)]
            fn div(self, rhs: Self) -> Self::Output {
                let lhs_u64: u64 = self.into();
                let rhs_u64: u64 = rhs.into();
                (lhs_u64 / rhs_u64).into()
            }
        }

        impl AddAssign for $currency_type {
            #[allow(clippy::arithmetic_side_effects)]
            fn add_assign(&mut self, rhs: Self) {
                *self = *self + rhs;
            }
        }

        impl SubAssign for $currency_type {
            #[allow(clippy::arithmetic_side_effects)]
            fn sub_assign(&mut self, rhs: Self) {
                *self = *self - rhs;
            }
        }

        impl MulAssign for $currency_type {
            #[allow(clippy::arithmetic_side_effects)]
            fn mul_assign(&mut self, rhs: Self) {
                *self = *self * rhs;
            }
        }

        impl DivAssign for $currency_type {
            #[allow(clippy::arithmetic_side_effects)]
            fn div_assign(&mut self, rhs: Self) {
                *self = *self / rhs;
            }
        }
    };
}

macro_rules! impl_approx {
    ($currency_type:ident) => {
        #[cfg(feature = "approx")]
        impl AbsDiffEq<Self> for $currency_type {
            type Epsilon = Self;

            fn default_epsilon() -> Self::Epsilon {
                u64::default_epsilon().into()
            }

            fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
                u64::abs_diff_eq(&u64::from(*self), &u64::from(*other), epsilon.into())
            }

            fn abs_diff_ne(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
                u64::abs_diff_ne(&u64::from(*self), &u64::from(*other), epsilon.into())
            }
        }
    };
}

pub trait Currency: ToFixed + Into<u64> + From<u64> + Clone + Copy {
    const MAX: Self;
    const ZERO: Self;

    fn is_zero(&self) -> bool {
        Into::<u64>::into(*self) == 0
    }

    fn to_u64(&self) -> u64 {
        (*self).into()
    }

    fn saturating_add(&self, rhv: Self) -> Self {
        Into::<u64>::into(*self).saturating_add(rhv.into()).into()
    }

    #[allow(clippy::arithmetic_side_effects)]
    fn saturating_div(&self, rhv: Self) -> Self {
        Into::<u64>::into(*self).saturating_div(rhv.into()).into()
    }

    fn saturating_sub(&self, rhv: Self) -> Self {
        Into::<u64>::into(*self).saturating_sub(rhv.into()).into()
    }

    fn saturating_mul(&self, rhv: Self) -> Self {
        Into::<u64>::into(*self).saturating_mul(rhv.into()).into()
    }
}

impl_arithmetic_operators!(AlphaCurrency);
impl_approx!(AlphaCurrency);
impl_currency_reqs!(AlphaCurrency);

impl_arithmetic_operators!(TaoCurrency);
impl_approx!(TaoCurrency);
impl_currency_reqs!(TaoCurrency);

impl Currency for AlphaCurrency {
    const MAX: Self = Self(u64::MAX);
    const ZERO: Self = Self(0);
}

impl Currency for TaoCurrency {
    const MAX: Self = Self(u64::MAX);
    const ZERO: Self = Self(0);
}
