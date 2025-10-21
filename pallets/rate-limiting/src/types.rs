use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::{pallet_prelude::DispatchError, traits::GetCallMetadata};
use scale_info::TypeInfo;

/// Resolves the optional context within which a rate limit applies.
pub trait RateLimitContextResolver<Call, Context> {
    /// Returns `Some(context)` when the limit should be applied per-context, or `None` for global
    /// limits.
    fn context(call: &Call) -> Option<Context>;
}

/// Identifies a runtime call by pallet and extrinsic indices.
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    Debug,
)]
pub struct TransactionIdentifier {
    /// Pallet variant index.
    pub pallet_index: u8,
    /// Call variant index within the pallet.
    pub extrinsic_index: u8,
}

impl TransactionIdentifier {
    /// Builds a new identifier from pallet/extrinsic indices.
    pub const fn new(pallet_index: u8, extrinsic_index: u8) -> Self {
        Self {
            pallet_index,
            extrinsic_index,
        }
    }

    /// Returns the pallet and extrinsic names associated with this identifier.
    pub fn names<T>(&self) -> Result<(&'static str, &'static str), DispatchError>
    where
        T: crate::pallet::Config,
        <T as crate::pallet::Config>::RuntimeCall: GetCallMetadata,
    {
        let modules = <T as crate::pallet::Config>::RuntimeCall::get_module_names();
        let pallet_name = modules
            .get(self.pallet_index as usize)
            .copied()
            .ok_or(crate::pallet::Error::<T>::InvalidRuntimeCall)?;
        let call_names = <T as crate::pallet::Config>::RuntimeCall::get_call_names(pallet_name);
        let extrinsic_name = call_names
            .get(self.extrinsic_index as usize)
            .copied()
            .ok_or(crate::pallet::Error::<T>::InvalidRuntimeCall)?;
        Ok((pallet_name, extrinsic_name))
    }

    /// Builds an identifier from a runtime call by extracting pallet/extrinsic indices.
    pub fn from_call<T>(
        call: &<T as crate::pallet::Config>::RuntimeCall,
    ) -> Result<Self, DispatchError>
    where
        T: crate::pallet::Config,
    {
        call.using_encoded(|encoded| {
            let pallet_index = *encoded
                .get(0)
                .ok_or(crate::pallet::Error::<T>::InvalidRuntimeCall)?;
            let extrinsic_index = *encoded
                .get(1)
                .ok_or(crate::pallet::Error::<T>::InvalidRuntimeCall)?;
            Ok(TransactionIdentifier::new(pallet_index, extrinsic_index))
        })
    }
}

/// Configuration value for a rate limit.
#[cfg_attr(feature = "std", derive(serde::Deserialize, serde::Serialize))]
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Encode,
    Decode,
    DecodeWithMemTracking,
    TypeInfo,
    MaxEncodedLen,
    Debug,
)]
pub enum RateLimit<BlockNumber> {
    /// Use the pallet-level default rate limit.
    Default,
    /// Apply an exact rate limit measured in blocks.
    Exact(BlockNumber),
}
