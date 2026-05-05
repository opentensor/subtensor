//! # Multi-Collective Pallet
//!
//! Stores the membership of one or more named collectives keyed by a
//! runtime-defined `CollectiveId`. Each collective is configured by a
//! `CollectivesInfo` impl: name, min/max members, optional term duration.
//!
//! ## Membership
//!
//! Members are kept sorted by `AccountId` in a per-collective `BoundedVec`.
//! Four extrinsics mutate the set, each gated by its own origin:
//! - [`Pallet::add_member`] (`T::AddOrigin`)
//! - [`Pallet::remove_member`] (`T::RemoveOrigin`)
//! - [`Pallet::swap_member`] (`T::SwapOrigin`)
//! - [`Pallet::set_members`] (`T::SetOrigin`)
//!
//! Every mutation fires `T::OnMembersChanged` with the incoming and
//! outgoing accounts.
//!
//! ## Rotations
//!
//! Collectives with `CollectiveInfo::term_duration = Some(d)` rotate on
//! schedule: `on_initialize` calls `T::OnNewTerm::on_new_term(id)` whenever
//! `block_number % d == 0`. The runtime-provided handler recomputes the
//! membership and pushes it back through `set_members`.
//!
//! [`Pallet::force_rotate`] (gated by `T::RotateOrigin`) triggers the same
//! hook on demand, for bootstrapping the first term or as a privileged
//! override.
//!
//! ## Inspection
//!
//! Other pallets read membership through [`CollectiveInspect`], implemented
//! by `Pallet<T>` over `Members<_>`.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use frame_support::{
    dispatch::DispatchResult,
    pallet_prelude::*,
    traits::{ChangeMembers, EnsureOriginWithArg},
};
use frame_system::pallet_prelude::*;
use num_traits::ops::checked::CheckedRem;
pub use pallet::*;
pub use subtensor_runtime_common::OnMembersChanged;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
pub mod weights;
pub use weights::WeightInfo;

pub const MAX_COLLECTIVE_NAME_LEN: usize = 32;
type CollectiveName = [u8; MAX_COLLECTIVE_NAME_LEN];

/// Pinned at 0 to satisfy try-runtime CLI's pre/post-upgrade checks. The
/// project tracks migrations via a per-pallet `HasMigrationRun` map (see
/// `pallet-crowdloan`), so this value is not bumped on schema changes.
pub const STORAGE_VERSION: frame_support::traits::StorageVersion =
    frame_support::traits::StorageVersion::new(0);

#[frame_support::pallet]
#[allow(clippy::expect_used)]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
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

        /// Required origin for setting the full member list of a collective.
        type SetOrigin: EnsureOriginWithArg<Self::RuntimeOrigin, Self::CollectiveId>;

        /// Required origin for `force_rotate`.
        type RotateOrigin: EnsureOriginWithArg<Self::RuntimeOrigin, Self::CollectiveId>;

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

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        /// Helper for setting up cross-pallet state needed by benchmarks.
        #[cfg(feature = "runtime-benchmarks")]
        type BenchmarkHelper: BenchmarkHelper<Self::CollectiveId>;
    }

    /// Benchmark setup helper. The runtime supplies a non-rotatable
    /// collective for member-management benchmarks and a rotatable one for
    /// `force_rotate`.
    #[cfg(feature = "runtime-benchmarks")]
    pub trait BenchmarkHelper<CollectiveId> {
        /// A collective whose `info.max_members` allows reaching `MaxMembers`
        /// and whose `info.min_members == 0`, so member-management
        /// benchmarks can fill and drain freely.
        fn collective() -> CollectiveId;
        /// A collective whose `CollectiveInfo::term_duration` is `Some`,
        /// for the `force_rotate` benchmark.
        fn rotatable_collective() -> CollectiveId;
    }

    /// Members of each collective, kept sorted by `AccountId`.
    ///
    /// The sorted invariant is maintained by every write path
    /// (`add_member`, `remove_member`, `swap_member`, `set_members`) so
    /// that membership lookups can use `binary_search` and `set_members`
    /// can diff against the previous set with a linear merge.
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
        MembersSet {
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
        /// `CollectiveInfo::term_duration` is `None`. Such collectives
        /// are managed directly via the membership extrinsics and have
        /// no rotation hook to trigger.
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
            Pallet::<T>::check_integrity();
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #![deny(clippy::expect_used)]

        #[pallet::call_index(0)]
        #[pallet::weight(
            T::WeightInfo::add_member().saturating_add(T::OnMembersChanged::weight())
        )]
        pub fn add_member(
            origin: OriginFor<T>,
            collective_id: T::CollectiveId,
            who: T::AccountId,
        ) -> DispatchResult {
            T::AddOrigin::ensure_origin(origin, &collective_id)?;
            let info = T::Collectives::info(collective_id).ok_or(Error::<T>::CollectiveNotFound)?;

            Members::<T>::try_mutate(collective_id, |members| -> DispatchResult {
                let pos = members
                    .binary_search(&who)
                    .err()
                    .ok_or(Error::<T>::AlreadyMember)?;
                if let Some(max) = info.max_members {
                    ensure!(members.len() < max as usize, Error::<T>::TooManyMembers);
                }
                members
                    .try_insert(pos, who.clone())
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
        #[pallet::weight(
            T::WeightInfo::remove_member().saturating_add(T::OnMembersChanged::weight())
        )]
        pub fn remove_member(
            origin: OriginFor<T>,
            collective_id: T::CollectiveId,
            who: T::AccountId,
        ) -> DispatchResult {
            T::RemoveOrigin::ensure_origin(origin, &collective_id)?;
            let info = T::Collectives::info(collective_id).ok_or(Error::<T>::CollectiveNotFound)?;

            Members::<T>::try_mutate(collective_id, |members| -> DispatchResult {
                let pos = members
                    .binary_search(&who)
                    .map_err(|_| Error::<T>::NotMember)?;
                ensure!(
                    members.len() > info.min_members as usize,
                    Error::<T>::TooFewMembers
                );
                members.remove(pos);
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
        #[pallet::weight(
            T::WeightInfo::swap_member().saturating_add(T::OnMembersChanged::weight())
        )]
        pub fn swap_member(
            origin: OriginFor<T>,
            collective_id: T::CollectiveId,
            remove: T::AccountId,
            add: T::AccountId,
        ) -> DispatchResult {
            T::SwapOrigin::ensure_origin(origin, &collective_id)?;
            T::Collectives::info(collective_id).ok_or(Error::<T>::CollectiveNotFound)?;

            Members::<T>::try_mutate(collective_id, |members| -> DispatchResult {
                let pos_remove = members
                    .binary_search(&remove)
                    .map_err(|_| Error::<T>::NotMember)?;
                let pos_add = members
                    .binary_search(&add)
                    .err()
                    .ok_or(Error::<T>::AlreadyMember)?;
                members.remove(pos_remove);
                // After removing index `pos_remove`, every position strictly
                // greater than it has shifted down by one. The branch guards
                // `pos_add >= 1`, so `saturating_sub` is exact here.
                let insert_at = if pos_remove < pos_add {
                    pos_add.saturating_sub(1)
                } else {
                    pos_add
                };
                members
                    .try_insert(insert_at, add.clone())
                    .map_err(|_| Error::<T>::TooManyMembers)?;
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
        #[pallet::weight(
            T::WeightInfo::set_members().saturating_add(T::OnMembersChanged::weight())
        )]
        pub fn set_members(
            origin: OriginFor<T>,
            collective_id: T::CollectiveId,
            members: Vec<T::AccountId>,
        ) -> DispatchResult {
            T::SetOrigin::ensure_origin(origin, &collective_id)?;
            let info = T::Collectives::info(collective_id).ok_or(Error::<T>::CollectiveNotFound)?;

            // Validate new member list
            ensure!(
                members.len() >= info.min_members as usize,
                Error::<T>::TooFewMembers
            );
            if let Some(max) = info.max_members {
                ensure!(members.len() <= max as usize, Error::<T>::TooManyMembers);
            }

            // Sort + dedup; the sorted form is what we store, so the
            // dedup pass and the storage write share the same buffer.
            let len_before = members.len();
            let mut sorted = members;
            sorted.sort();
            sorted.dedup();
            ensure!(sorted.len() == len_before, Error::<T>::DuplicateAccounts);

            let old_members = Members::<T>::get(collective_id);
            let bounded =
                BoundedVec::try_from(sorted.clone()).map_err(|_| Error::<T>::TooManyMembers)?;
            Members::<T>::insert(collective_id, bounded);

            let (incoming, outgoing) =
                <() as ChangeMembers<T::AccountId>>::compute_members_diff_sorted(
                    &sorted,
                    &old_members,
                );

            T::OnMembersChanged::on_members_changed(collective_id, &incoming, &outgoing);
            Self::deposit_event(Event::MembersSet {
                collective_id,
                members: sorted,
            });
            Ok(())
        }

        /// Manually trigger the `OnNewTerm` hook for `collective_id`,
        /// outside of the natural `n % term_duration == 0` schedule in
        /// `on_initialize`. Used for the very first population (the
        /// natural rotation only fires after the first term boundary,
        /// which can be days or months in) and as a privileged override
        /// during incidents.
        ///
        /// Restricted to collectives whose `CollectiveInfo::term_duration`
        /// is `Some(_)`. Curated collectives (Triumvirate, Proposers) are
        /// managed directly via `add_member` / `remove_member` /
        /// `swap_member` / `set_members` and have no rotation hook, so
        /// refusing the call here surfaces a misconfigured rotate
        /// extrinsic as `CollectiveDoesNotRotate` instead of silently
        /// consuming weight.
        #[pallet::call_index(4)]
        #[pallet::weight(
            T::WeightInfo::force_rotate().saturating_add(T::OnNewTerm::weight())
        )]
        pub fn force_rotate(
            origin: OriginFor<T>,
            collective_id: T::CollectiveId,
        ) -> DispatchResult {
            T::RotateOrigin::ensure_origin(origin, &collective_id)?;
            let info = T::Collectives::info(collective_id).ok_or(Error::<T>::CollectiveNotFound)?;
            ensure!(
                info.term_duration.is_some(),
                Error::<T>::CollectiveDoesNotRotate
            );
            // The hook returns `Weight` so `on_initialize` can accumulate
            // actual block weight; `force_rotate` is Root-only and just
            // pays the worst-case bound, no refund.
            let _ = T::OnNewTerm::on_new_term(collective_id);
            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    /// Validates the `CollectivesInfo` configuration against the
    /// pallet's storage cap. Called from the `integrity_test` hook
    /// at construction; extracted so tests can drive it directly.
    ///
    /// Guards against `CollectiveInfo` / `T::MaxMembers` mismatch: a
    /// runtime declaring `max_members` (or `min_members`) greater
    /// than `T::MaxMembers` would pass the per-collective cap check
    /// in `add_member` / `set_members` but then fail the `BoundedVec`
    /// bound with a confusing `TooManyMembers` at the storage
    /// ceiling. Failing construction here makes the inconsistent
    /// config unreachable at runtime.
    ///
    /// Alternative structural fix (not taken): drop `max_members`
    /// from `CollectiveInfo` and expose it via a per-collective
    /// method on `CollectivesInfo` computed against `T::MaxMembers`
    /// (e.g. `fn max_members_of(id) -> u32`). That eliminates the
    /// field mismatch by construction at the cost of a
    /// `CollectivesInfo` trait-shape change.
    pub fn check_integrity() {
        let storage_max = T::MaxMembers::get();
        for collective in T::Collectives::collectives() {
            let info = collective.info;

            assert!(
                info.min_members <= storage_max,
                "CollectiveInfo::min_members ({}) exceeds T::MaxMembers ({}); collective cannot reach its min",
                info.min_members,
                storage_max,
            );

            if let Some(max) = info.max_members {
                assert!(
                    max <= storage_max,
                    "CollectiveInfo::max_members ({}) exceeds T::MaxMembers ({}); storage cannot hold this many",
                    max,
                    storage_max,
                );
                assert!(
                    info.min_members <= max,
                    "CollectiveInfo::min_members ({}) exceeds max_members ({}); collective is unreachable",
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

/// Handler for when a new term of a collective has started.
pub trait OnNewTerm<CollectiveId> {
    /// A new term of a collective has started. Returns the actual weight
    /// consumed so `on_initialize` can accumulate per-block hook weight
    /// across all rotating collectives.
    fn on_new_term(collective_id: CollectiveId) -> Weight;
    /// Worst-case upper bound on `on_new_term`'s weight, used to
    /// pre-charge `force_rotate`.
    fn weight() -> Weight;
}

#[impl_trait_for_tuples::impl_for_tuples(10)]
impl<CollectiveId: Clone> OnNewTerm<CollectiveId> for Tuple {
    // `for_tuples!` mutates `weight` inline; clippy can't see the expansion.
    #[allow(clippy::let_and_return)]
    fn on_new_term(collective_id: CollectiveId) -> Weight {
        let mut weight = Weight::zero();
        for_tuples!( #( weight = weight.saturating_add(Tuple::on_new_term(collective_id.clone())); )* );
        weight
    }

    fn weight() -> Weight {
        #[allow(clippy::let_and_return)]
        let mut weight = Weight::zero();
        for_tuples!( #( weight.saturating_accrue(Tuple::weight()); )* );
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
        Members::<T>::get(collective_id).binary_search(who).is_ok()
    }
    fn member_count(collective_id: T::CollectiveId) -> u32 {
        Members::<T>::get(collective_id).len() as u32
    }
}
