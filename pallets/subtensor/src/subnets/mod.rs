use super::*;
use codec::{Decode, Encode, MaxEncodedLen};
pub mod lock;
pub mod registration;
pub mod serving;
pub mod subnet;
pub mod tempo;
pub mod uids;
pub mod weights;

#[derive(
    Clone,
    Copy,
    Debug,
    Decode,
    Default,
    Encode,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    MaxEncodedLen,
    scale_info::TypeInfo,
)]
pub enum Mechanism {
    #[default]
    Stable = 0,
    Dynamic = 1,
}

impl Mechanism {
    pub fn is_stable(&self) -> bool {
        matches!(self, Self::Stable)
    }

    pub fn is_dynamic(&self) -> bool {
        matches!(self, Self::Dynamic)
    }
}
