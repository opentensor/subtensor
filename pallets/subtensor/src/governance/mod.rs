use super::*;

pub mod eligibility;

/// Notification fired when a coldkey's root-registered status flips.
///
/// `on_added` runs the first time a coldkey acquires a root hotkey
/// (`RootRegisteredHotkeyCount` transitions 0 to 1). `on_removed` runs
/// when it loses its last root hotkey (transitions back to 0). Pure
/// 0↔1 edges: increments past 1 and decrements above 1 are silent.
pub trait OnRootRegistrationChange<AccountId> {
    fn on_added(coldkey: &AccountId);
    fn on_removed(coldkey: &AccountId);
}

impl<AccountId> OnRootRegistrationChange<AccountId> for () {
    fn on_added(_: &AccountId) {}
    fn on_removed(_: &AccountId) {}
}

/// Read-side accessor used by `try_state` to verify that the
/// `EconomicEligible` collective stays in sync with the set of coldkeys
/// holding at least one root-registered hotkey.
///
/// Returning `None` skips the cross-pallet check (test mocks that do
/// not wire up `pallet-multi-collective`).
pub trait EconomicEligibleInspector<AccountId> {
    fn members() -> Option<alloc::vec::Vec<AccountId>>;
}

impl<AccountId> EconomicEligibleInspector<AccountId> for () {
    fn members() -> Option<alloc::vec::Vec<AccountId>> {
        None
    }
}
