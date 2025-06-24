use core::fmt::{self, Display, Formatter};
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

#[cfg(feature = "approx")]
use approx::AbsDiffEq;
use codec::{Compact, CompactAs, Decode, Encode, Error as CodecError, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use substrate_fixed::traits::{Fixed, ToFixed};
use subtensor_macros::freeze_struct;

#[freeze_struct("597e376f01cf675a")]
#[repr(transparent)]
#[derive(
    Deserialize,
    Serialize,
    Clone,
    Copy,
    Decode,
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
pub struct Alpha(u64);

impl TypeInfo for Alpha {
    type Identity = <u64 as TypeInfo>::Identity;
    fn type_info() -> scale_info::Type {
        <u64 as TypeInfo>::type_info()
    }
}

impl Display for Alpha {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl CompactAs for Alpha {
    type As = u64;

    fn encode_as(&self) -> &Self::As {
        &self.0
    }

    fn decode_from(v: Self::As) -> Result<Self, CodecError> {
        Ok(Self(v))
    }
}

impl From<Compact<Alpha>> for Alpha {
    fn from(c: Compact<Alpha>) -> Self {
        c.0
    }
}

impl From<Alpha> for u64 {
    fn from(val: Alpha) -> Self {
        val.0
    }
}

impl From<u64> for Alpha {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl ToFixed for Alpha {
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

impl Currency for Alpha {
    const MAX: Self = Self(u64::MAX);
    const ZERO: Self = Self(0);
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

macro_rules! impl_arithmetic_operators {
    ($currency_type:ident) => {
        impl Add for $currency_type {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                let lhs_u64: u64 = self.into();
                let rhs_u64: u64 = rhs.into();
                (lhs_u64 + rhs_u64).into()
            }
        }

        impl Sub for $currency_type {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                let lhs_u64: u64 = self.into();
                let rhs_u64: u64 = rhs.into();
                (lhs_u64 - rhs_u64).into()
            }
        }

        impl Mul for $currency_type {
            type Output = Self;

            fn mul(self, rhs: Self) -> Self::Output {
                let lhs_u64: u64 = self.into();
                let rhs_u64: u64 = rhs.into();
                (lhs_u64 * rhs_u64).into()
            }
        }

        impl Div for $currency_type {
            type Output = Self;

            fn div(self, rhs: Self) -> Self::Output {
                let lhs_u64: u64 = self.into();
                let rhs_u64: u64 = rhs.into();
                (lhs_u64 / rhs_u64).into()
            }
        }

        impl AddAssign for $currency_type {
            fn add_assign(&mut self, rhs: Self) {
                *self = *self + rhs;
            }
        }

        impl SubAssign for $currency_type {
            fn sub_assign(&mut self, rhs: Self) {
                *self = *self - rhs;
            }
        }

        impl MulAssign for $currency_type {
            fn mul_assign(&mut self, rhs: Self) {
                *self = *self * rhs;
            }
        }

        impl DivAssign for $currency_type {
            fn div_assign(&mut self, rhs: Self) {
                *self = *self / rhs;
            }
        }
    };
}

impl_arithmetic_operators!(Alpha);

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

impl_approx!(Alpha);
