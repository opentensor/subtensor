#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use frame_support::{dispatch::DispatchResult, pallet_prelude::*, traits::EnsureOriginWithArg};
use frame_system::pallet_prelude::*;
use num_traits::ops::checked::CheckedRem;
pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub const MAX_COLLECTIVE_NAME_LEN: usize = 32;
type CollectiveName = [u8; MAX_COLLECTIVE_NAME_LEN];

#[frame_support::pallet(dev_mode)]
#[allow(clippy::expect_used)]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type CollectiveId: Parameter + MaxEncodedLen + Copy + CanRotate;

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
    pub enum Event<T: Config> {
        MemberAdded {
            collective_id: T::CollectiveId,
            who: T::AccountId,
        },
        MemberRemoved {
            collective_id: T::CollectiveId,
            who: T::AccountId,
        },
        MemberSwapped {
            collective_id: T::CollectiveId,
            removed: T::AccountId,
            added: T::AccountId,
        },
        MembersReset {
            collective_id: T::CollectiveId,
            members: Vec<T::AccountId>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Account is already a member of this collective.
        AlreadyMember,
        /// Account is not a member of this collective.
        NotMember,
        /// Adding a member would exceed the maximum for this collective.
        TooManyMembers,
        /// Removing a member would go below the minimum for this collective.
        TooFewMembers,
        /// The collective is not recognized.
        CollectiveNotFound,
        /// Duplicate accounts in member list.
        DuplicateAccounts,
        /// `force_rotate` was called for a collective whose
        /// `CollectiveId::can_rotate()` is false. Such collectives are
        /// managed by Root directly via the membership extrinsics and
        /// have no rotation hook to trigger.
        CollectiveDoesNotRotate,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_initialize(n: BlockNumberFor<T>) -> Weight {
            let mut weight = Weight::zero();

            for collective in T::Collectives::collectives() {
                // Conservative upper bound for the iteration cost. Matches the
                // storage-backed case; static `CollectivesInfo` impls pay a
                // smaller CPU cost, so this is a safe overestimate.
                weight.saturating_accrue(T::DbWeight::get().reads(1));

                if collective
                    .info
                    .term_duration
                    .is_some_and(|td| n.checked_rem(&td).unwrap_or(n).is_zero())
                {
                    weight.saturating_accrue(T::OnNewTerm::on_new_term(collective.id));
                }
            }

            weight
        }

        fn integrity_test() {
            // Guards against `CollectiveInfo` / `T::MaxMembers` mismatch: a runtime
            // declaring `max_members` (or `min_members`) greater than
            // `T::MaxMembers` would pass the per-collective cap check in
            // `add_member` / `reset_members` but then fail the `BoundedVec` bound
            // with a confusing `TooManyMembers` at the storage ceiling. Failing
            // construction here makes the inconsistent config unreachable at
            // runtime.
            //
            // Alternative structural fix (not taken): drop `max_members` from
            // `CollectiveInfo` and expose it via a per-collective method on
            // `CollectivesInfo` computed against `T::MaxMembers` (e.g.
            // `fn max_members_of(id) -> u32`). That eliminates the field mismatch
            // by construction at the cost of a `CollectivesInfo` trait-shape change.
            let storage_max = T::MaxMembers::get();
            for collective in T::Collectives::collectives() {
                let info = collective.info;

                assert!(
                    info.min_members <= storage_max,
                    "CollectiveInfo::min_members ({}) exceeds T::MaxMembers ({}) — collective cannot reach its min",
                    info.min_members,
                    storage_max,
                );

                if let Some(max) = info.max_members {
                    assert!(
                        max <= storage_max,
                        "CollectiveInfo::max_members ({}) exceeds T::MaxMembers ({}) — storage cannot hold this many",
                        max,
                        storage_max,
                    );
                    assert!(
                        info.min_members <= max,
                        "CollectiveInfo::min_members ({}) exceeds max_members ({}) — collective is unreachable",
                        info.min_members,
                        max,
                    );
                }

                // `Some(0)` for term_duration is indistinguishable from "rotate
                // every block" at the type level, but the `n % td` check in
                // `on_initialize` short-circuits via `checked_rem` and never
                // fires. Reject it here rather than let a misconfigured runtime
                // silently disable rotations. Use `None` to opt out.
                if let Some(td) = info.term_duration {
                    assert!(
                        !td.is_zero(),
                        "CollectiveInfo::term_duration = Some(0) silently disables rotations; use None to opt out",
                    );
                }
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        pub fn add_member(
            origin: OriginFor<T>,
            collective_id: T::CollectiveId,
            who: T::AccountId,
        ) -> DispatchResult {
            T::AddOrigin::ensure_origin(origin, &collective_id)?;
            let info = T::Collectives::info(collective_id).ok_or(Error::<T>::CollectiveNotFound)?;

            Members::<T>::try_mutate(collective_id, |members| -> DispatchResult {
                ensure!(!members.contains(&who), Error::<T>::AlreadyMember);
                if let Some(max) = info.max_members {
                    ensure!(members.len() < max as usize, Error::<T>::TooManyMembers);
                }
                members
                    .try_push(who.clone())
                    .map_err(|_| Error::<T>::TooManyMembers)?;
                Ok(())
            })?;

            T::OnMembersChanged::on_members_changed(
                collective_id,
                core::slice::from_ref(&who),
                &[],
            );
            Self::deposit_event(Event::MemberAdded { collective_id, who });
            Ok(())
        }

        #[pallet::call_index(1)]
        pub fn remove_member(
            origin: OriginFor<T>,
            collective_id: T::CollectiveId,
            who: T::AccountId,
        ) -> DispatchResult {
            T::RemoveOrigin::ensure_origin(origin, &collective_id)?;
            let info = T::Collectives::info(collective_id).ok_or(Error::<T>::CollectiveNotFound)?;

            Members::<T>::try_mutate(collective_id, |members| -> DispatchResult {
                ensure!(members.contains(&who), Error::<T>::NotMember);
                ensure!(
                    members.len() > info.min_members as usize,
                    Error::<T>::TooFewMembers
                );
                members.retain(|m| m != &who);
                Ok(())
            })?;

            T::OnMembersChanged::on_members_changed(
                collective_id,
                &[],
                core::slice::from_ref(&who),
            );
            Self::deposit_event(Event::MemberRemoved { collective_id, who });
            Ok(())
        }

        #[pallet::call_index(2)]
        pub fn swap_member(
            origin: OriginFor<T>,
            collective_id: T::CollectiveId,
            remove: T::AccountId,
            add: T::AccountId,
        ) -> DispatchResult {
            T::SwapOrigin::ensure_origin(origin, &collective_id)?;
            T::Collectives::info(collective_id).ok_or(Error::<T>::CollectiveNotFound)?;

            Members::<T>::try_mutate(collective_id, |members| -> DispatchResult {
                let pos = members
                    .iter()
                    .position(|m| m == &remove)
                    .ok_or(Error::<T>::NotMember)?;
                ensure!(!members.contains(&add), Error::<T>::AlreadyMember);
                *members.get_mut(pos).ok_or(Error::<T>::NotMember)? = add.clone();
                Ok(())
            })?;

            T::OnMembersChanged::on_members_changed(
                collective_id,
                core::slice::from_ref(&add),
                core::slice::from_ref(&remove),
            );
            Self::deposit_event(Event::MemberSwapped {
                collective_id,
                removed: remove,
                added: add,
            });
            Ok(())
        }

        #[pallet::call_index(3)]
        pub fn reset_members(
            origin: OriginFor<T>,
            collective_id: T::CollectiveId,
            members: Vec<T::AccountId>,
        ) -> DispatchResult {
            T::ResetOrigin::ensure_origin(origin, &collective_id)?;
            let info = T::Collectives::info(collective_id).ok_or(Error::<T>::CollectiveNotFound)?;

            // Validate new member list
            ensure!(
                members.len() >= info.min_members as usize,
                Error::<T>::TooFewMembers
            );
            if let Some(max) = info.max_members {
                ensure!(members.len() <= max as usize, Error::<T>::TooManyMembers);
            }

            // Check for duplicates
            let mut sorted = members.clone();
            sorted.sort();
            sorted.dedup();
            ensure!(sorted.len() == members.len(), Error::<T>::DuplicateAccounts);

            let old_members = Members::<T>::get(collective_id);
            let bounded =
                BoundedVec::try_from(members.clone()).map_err(|_| Error::<T>::TooManyMembers)?;
            Members::<T>::insert(collective_id, bounded);

            // Compute incoming/outgoing
            let incoming: Vec<_> = members
                .iter()
                .filter(|m| !old_members.contains(m))
                .cloned()
                .collect();
            let outgoing: Vec<_> = old_members
                .iter()
                .filter(|m| !members.contains(m))
                .cloned()
                .collect();

            T::OnMembersChanged::on_members_changed(collective_id, &incoming, &outgoing);
            Self::deposit_event(Event::MembersReset {
                collective_id,
                members,
            });
            Ok(())
        }

        /// Manually trigger the `OnNewTerm` hook for `collective_id`,
        /// outside of the natural `n % term_duration == 0` schedule in
        /// `on_initialize`. Used for the very first population (the
        /// natural rotation only fires after the first term boundary,
        /// which can be days or months in) and as a Root override
        /// during incidents.
        ///
        /// Restricted to collectives whose `CollectiveId::can_rotate()`
        /// is true. Curated collectives (Triumvirate, Proposers) are
        /// managed directly via `add_member` / `remove_member` /
        /// `swap_member` / `reset_members` and have no rotation hook
        /// — refusing the call here surfaces a misconfigured Root
        /// extrinsic as `CollectiveDoesNotRotate` instead of silently
        /// consuming weight.
        ///
        /// Origin: Root.
        #[pallet::call_index(4)]
        pub fn force_rotate(
            origin: OriginFor<T>,
            collective_id: T::CollectiveId,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            ensure!(
                collective_id.can_rotate(),
                Error::<T>::CollectiveDoesNotRotate
            );
            // Existence check after the rotatability gate, so a typo'd
            // id still surfaces `CollectiveNotFound` if it was meant to
            // be rotatable.
            T::Collectives::info(collective_id).ok_or(Error::<T>::CollectiveNotFound)?;
            let weight = T::OnNewTerm::on_new_term(collective_id);
            Ok(Some(weight).into())
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

/// Whether a `CollectiveId` represents a rotatable collective. Implemented
/// by the runtime on its concrete `CollectiveId` enum and consumed by
/// `force_rotate` to refuse calls for collectives that have no rotation
/// source (e.g. Triumvirate / Proposers — managed by Root directly).
///
/// Kept as a property of the *id* rather than `CollectiveInfo` so the
/// rotatability of each collective is documented at the variant
/// definition site, not in a separate config table.
pub trait CanRotate {
    fn can_rotate(&self) -> bool;
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

#[impl_trait_for_tuples::impl_for_tuples(10)]
impl<CollectiveId: Clone, AccountId> OnMembersChanged<CollectiveId, AccountId> for Tuple {
    fn on_members_changed(
        collective_id: CollectiveId,
        incoming: &[AccountId],
        outgoing: &[AccountId],
    ) {
        for_tuples!( #( Tuple::on_members_changed(collective_id.clone(), incoming, outgoing); )* );
    }
}

/// Handler for when a new term of a collective has started.
pub trait OnNewTerm<CollectiveId> {
    /// A new term of a collective has started.
    fn on_new_term(collective_id: CollectiveId) -> Weight;
}

#[impl_trait_for_tuples::impl_for_tuples(10)]
impl<CollectiveId: Clone> OnNewTerm<CollectiveId> for Tuple {
    fn on_new_term(collective_id: CollectiveId) -> Weight {
        let mut weight = Weight::zero();
        for_tuples!( #( weight = weight.saturating_add(Tuple::on_new_term(collective_id.clone())); )* );
        weight
    }
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

impl<T: Config> CollectiveInspect<T::AccountId, T::CollectiveId> for Pallet<T> {
    fn members_of(collective_id: T::CollectiveId) -> Vec<T::AccountId> {
        Members::<T>::get(collective_id).to_vec()
    }
    fn is_member(collective_id: T::CollectiveId, who: &T::AccountId) -> bool {
        Members::<T>::get(collective_id).contains(who)
    }
    fn member_count(collective_id: T::CollectiveId) -> u32 {
        Members::<T>::get(collective_id).len() as u32
    }
}
