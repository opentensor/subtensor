use super::*;
use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::weights::Weight;
use scale_info::TypeInfo;
use substrate_fixed::types::U64F64;

pub mod ema;
pub mod ref_count;

/// Per-coldkey EMA state.
#[derive(
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Debug,
    Encode,
    Decode,
    DecodeWithMemTracking,
    MaxEncodedLen,
    TypeInfo,
)]
pub struct EmaState {
    /// Current EMA value.
    pub ema: U64F64,
    /// Number of samples folded into `ema`.
    pub samples: u32,
}

/// Hook for coldkey root-registration transitions.
pub trait OnRootRegistrationChange<AccountId> {
    /// Called when `coldkey` enters the root-registered set.
    fn on_added(coldkey: &AccountId);
    /// Called when `coldkey` leaves the root-registered set.
    fn on_removed(coldkey: &AccountId);
}

impl<AccountId> OnRootRegistrationChange<AccountId> for () {
    fn on_added(_: &AccountId) {}
    fn on_removed(_: &AccountId) {}
}

/// Snapshot of the root-registered coldkey set.
pub trait RootRegisteredInspector<AccountId> {
    /// Returns the current snapshot, or `None` if unavailable.
    fn members() -> Option<alloc::vec::Vec<AccountId>>;
}

impl<AccountId> RootRegisteredInspector<AccountId> for () {
    fn members() -> Option<alloc::vec::Vec<AccountId>> {
        None
    }
}

/// Computes a coldkey's next stake EMA value.
pub trait EmaStrategy<AccountId> {
    /// Returns the new EMA for `coldkey` given its `previous` state,
    /// paired with the actual weight consumed by the call. The sample
    /// counter on `previous` is the count *before* this tick, so a
    /// brand-new entry arrives with `samples == 0`.
    fn next(coldkey: &AccountId, previous: EmaState) -> (U64F64, Weight);
    /// Worst-case weight of `next`.
    fn weight() -> Weight;
}

/// Freezes the EMA at its previous value. Default for runtimes /
/// test mocks that don't compute EMAs.
impl<AccountId> EmaStrategy<AccountId> for () {
    fn next(_: &AccountId, previous: EmaState) -> (U64F64, Weight) {
        (previous.ema, Weight::zero())
    }

    fn weight() -> Weight {
        Weight::zero()
    }
}
