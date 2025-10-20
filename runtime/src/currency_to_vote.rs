//! An implementation of `CurrencyToVote` tailored for chain's that have a balance type of u64.
//!
//! The factor is the `(total_issuance / u64::MAX).max(1)`, represented as u64.

use sp_staking::currency_to_vote::CurrencyToVote;

pub struct SubtensorCurrencyToVote;

impl CurrencyToVote<u64> for SubtensorCurrencyToVote {
    /// Vote value independent of issuance.
    fn to_vote(value: u64, _issuance: u64) -> u64 {
        value
    }

    /// Our currency maxes out at u64, so this will never saturate.
    fn to_currency(value: u128, _issuance: u64) -> u64 {
        value.try_into().unwrap_or(u64::MAX)
    }

    /// Issuance will never by > u64, so will never will downscale.
    fn will_downscale(_issuance: u64) -> Option<bool> {
        None
    }
}
