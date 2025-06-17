use core::fmt::{self, Display, Formatter};

use codec::{Compact, CompactAs, Decode, Encode, Error as CodecError, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
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

