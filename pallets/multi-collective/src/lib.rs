#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::EnsureOriginWithArg};
use frame_system::pallet_prelude::*;
use num_traits::ops::checked::CheckedRem;
pub use pallet::*;

pub const MAX_COLLECTIVE_NAME_LEN: usize = 32;
type CollectiveName = [u8; MAX_COLLECTIVE_NAME_LEN];

#[frame_support::pallet(dev_mode)]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type CollectiveId: Parameter + MaxEncodedLen + Copy;

        /// Provides per-collective information.
        type Collectives: CollectivesInfo<BlockNumberFor<Self>, CollectiveName, Id = Self::CollectiveId>;

        /// Required origin for adding a member to a collective.
        type AddOrigin: EnsureOriginWithArg<Self::RuntimeOrigin, Self::CollectiveId>;

        /// Required origin for removing a member from a collective.
        type RemoveOrigin: EnsureOriginWithArg<Self::RuntimeOrigin, Self::CollectiveId>;

        /// Required origin for swapping a member in a collective.
        type SwapOrigin: EnsureOriginWithArg<Self::RuntimeOrigin, Self::CollectiveId>;

        /// Required origin for resetting the members of a collective.
        type ResetOrigin: EnsureOriginWithArg<Self::RuntimeOrigin, Self::CollectiveId>;

        /// The receiver of the signal for when the members of a collective have changed.
        type OnMembersChanged: OnMembersChanged<Self::CollectiveId, Self::AccountId>;

        /// The receiver of the signal for when a new term of a collective has started.
        type OnNewTerm: OnNewTerm<Self::CollectiveId>;

        /// The maximum number of members per collective.
        ///
        /// This is used for benchmarking. Re-run the benchmarks if this changes.
        ///
        /// This is enforced in the code; the membership size can not exceed this limit.
        #[pallet::constant]
        type MaxMembers: Get<u32>;
    }

    #[pallet::storage]
    pub(super) type Members<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::CollectiveId,
        BoundedVec<T::AccountId, T::MaxMembers>,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {}

    #[pallet::error]
    pub enum Error<T> {}

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            let mut weight = Weight::zero();

            for collective in T::Collectives::collectives() {
                if let Some(term_duration) = collective.info.term_duration {
                    if n.checked_rem(&term_duration).unwrap_or(n).is_zero() {
                        weight.saturating_accrue(T::OnNewTerm::on_new_term(collective.id));
                    }
                }
            }

            weight
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        pub fn add_member(
            _origin: OriginFor<T>,
            _collective_id: T::CollectiveId,
            _who: T::AccountId,
        ) -> DispatchResult {
            Ok(())
        }

        #[pallet::call_index(1)]
        pub fn remove_member(
            _origin: OriginFor<T>,
            _collective_id: T::CollectiveId,
            _who: T::AccountId,
        ) -> DispatchResult {
            Ok(())
        }

        #[pallet::call_index(2)]
        pub fn swap_member(
            _origin: OriginFor<T>,
            _collective_id: T::CollectiveId,
            _remove: T::AccountId,
            _add: T::AccountId,
        ) -> DispatchResult {
            Ok(())
        }

        #[pallet::call_index(3)]
        pub fn reset_members(
            _origin: OriginFor<T>,
            _collective_id: T::CollectiveId,
            _members: Vec<T::AccountId>,
        ) -> DispatchResult {
            Ok(())
        }
    }
}

// Detailed information about a collective.
pub struct CollectiveInfo<Moment, Name> {
    pub name: Name,
    /// Minimum number of members for a collective.
    pub min_members: u32,
    /// Maximum number of members for a collective.
    pub max_members: Option<u32>,
    /// The duration of the term for a collective.
    pub term_duration: Option<Moment>,
}

/// Collective groups the information of a collective with its corresponding identifier.
pub struct Collective<Id, Moment, Name> {
    /// Identifier of the collective.
    pub id: Id,
    /// Information about the collective.
    pub info: CollectiveInfo<Moment, Name>,
}

/// Information on the collectives.
pub trait CollectivesInfo<Moment, Name> {
    /// The identifier for a collective.
    type Id: Parameter + MaxEncodedLen + Copy + Ord + PartialOrd + Send + Sync + 'static;

    /// Return the sorted iterable list of known collectives.
    fn collectives() -> impl Iterator<Item = Collective<Self::Id, Moment, Name>>;

    /// Return the list of identifiers of the known collectives.
    fn collective_ids() -> impl Iterator<Item = Self::Id> {
        Self::collectives().map(|c| c.id)
    }

    /// Return the collective info for collective `id`, by default this just looks it up in `Self::collectives()`.
    fn info(id: Self::Id) -> Option<CollectiveInfo<Moment, Name>> {
        Self::collectives().find(|c| c.id == id).map(|c| c.info)
    }
}

/// Handler for when the members of a collective have changed.
pub trait OnMembersChanged<CollectiveId, AccountId> {
    /// A collective's members have changed, `incoming` members have joined and
    /// `outgoing` members have left.
    fn on_members_changed(
        collective_id: CollectiveId,
        incoming: &[AccountId],
        outgoing: &[AccountId],
    );
}

/// Handler for when a new term of a collective has started.
pub trait OnNewTerm<CollectiveId> {
    /// A new term of a collective has started.
    fn on_new_term(collective_id: CollectiveId) -> Weight;
}

/// Trait for inspecting a collective.
pub trait CollectiveInspect<AccountId, CollectiveId> {
    /// Return the members of a collective.
    fn members_of(collective_id: CollectiveId) -> Vec<AccountId>;
    /// Return true if an account is a member of a collective.
    fn is_member(collective_id: CollectiveId, who: &AccountId) -> bool;
    /// Return the number of members of a collective.
    fn member_count(collective_id: CollectiveId) -> u32;
}
