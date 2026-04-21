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
    }

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
            origin: OriginFor<T>,
            collective_id: T::CollectiveId,
            who: T::AccountId,
        ) -> DispatchResult {
            T::AddOrigin::ensure_origin(origin, &collective_id)?;
            let info = T::Collectives::info(collective_id).ok_or(Error::<T>::CollectiveNotFound)?;

            Members::<T>::try_mutate(&collective_id, |members| -> DispatchResult {
                ensure!(!members.contains(&who), Error::<T>::AlreadyMember);
                if let Some(max) = info.max_members {
                    ensure!(members.len() < max as usize, Error::<T>::TooManyMembers);
                }
                members
                    .try_push(who.clone())
                    .map_err(|_| Error::<T>::TooManyMembers)?;
                Ok(())
            })?;

            T::OnMembersChanged::on_members_changed(collective_id, &[who.clone()], &[]);
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

            Members::<T>::try_mutate(&collective_id, |members| -> DispatchResult {
                ensure!(members.contains(&who), Error::<T>::NotMember);
                ensure!(
                    members.len() > info.min_members as usize,
                    Error::<T>::TooFewMembers
                );
                members.retain(|m| m != &who);
                Ok(())
            })?;

            T::OnMembersChanged::on_members_changed(collective_id, &[], &[who.clone()]);
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

            Members::<T>::try_mutate(&collective_id, |members| -> DispatchResult {
                let pos = members
                    .iter()
                    .position(|m| m == &remove)
                    .ok_or(Error::<T>::NotMember)?;
                ensure!(!members.contains(&add), Error::<T>::AlreadyMember);
                members[pos] = add.clone();
                Ok(())
            })?;

            T::OnMembersChanged::on_members_changed(
                collective_id,
                &[add.clone()],
                &[remove.clone()],
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

            let old_members = Members::<T>::get(&collective_id);
            let bounded =
                BoundedVec::try_from(members.clone()).map_err(|_| Error::<T>::TooManyMembers)?;
            Members::<T>::insert(&collective_id, bounded);

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

impl<T: Config> CollectiveInspect<T::AccountId, T::CollectiveId> for Pallet<T> {
    fn members_of(collective_id: T::CollectiveId) -> Vec<T::AccountId> {
        Members::<T>::get(&collective_id).to_vec()
    }
    fn is_member(collective_id: T::CollectiveId, who: &T::AccountId) -> bool {
        Members::<T>::get(&collective_id).contains(who)
    }
    fn member_count(collective_id: T::CollectiveId) -> u32 {
        Members::<T>::get(&collective_id).len() as u32
    }
}
