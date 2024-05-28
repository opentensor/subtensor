// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Collective system: Members of a set of account IDs can make their collective feelings known
//! through dispatched calls from one of two specialized origins.
//!
//! The membership can be provided in one of two ways: either directly, using the Root-dispatchable
//! function `set_members`, or indirectly, through implementing the `ChangeMembers`.
//! The pallet assumes that the amount of members stays at or below `MaxMembers` for its weight
//! calculations, but enforces this neither in `set_members` nor in `change_members_sorted`.
//!
//! A "prime" member may be set to help determine the default vote behavior based on chain
//! config. If `PrimeDefaultVote` is used, the prime vote acts as the default vote in case of any
//! abstentions after the voting period. If `MoreThanMajorityThenPrimeDefaultVote` is used, then
//! abstentions will first follow the majority of the collective voting, and then the prime
//! member.
//!
//! Voting happens through motions comprising a proposal (i.e. a curried dispatchable) plus a
//! number of approvals required for it to pass and be called. Motions are open for members to
//! vote on for a minimum period given by `MotionDuration`. As soon as the needed number of
//! approvals is given, the motion is closed and executed. If the number of approvals is not reached
//! during the voting period, then `close` may be called by any account in order to force the end
//! the motion explicitly. If a prime member is defined then their vote is used in place of any
//! abstentions and the proposal is executed if there are enough approvals counting the new votes.
//!
//! If there are not, or if no prime is set, then the motion is dropped without being executed.

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "128"]

use scale_info::TypeInfo;
use sp_io::storage;
use sp_runtime::{traits::Hash, BoundedVec, Permill, RuntimeDebug};
use sp_std::{marker::PhantomData, prelude::*, result};

use frame_support::{
    codec::{Decode, Encode, MaxEncodedLen},
    dispatch::{
        DispatchError, DispatchResultWithPostInfo, Dispatchable, GetDispatchInfo, Pays,
        PostDispatchInfo,
    },
    ensure,
    traits::{
        Backing, ChangeMembers, Defensive, DefensiveResult, EnsureOrigin, Get, GetBacking,
        InitializeMembers, StorageVersion,
    },
    weights::Weight,
};

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

pub use pallet::*;
pub use weights::WeightInfo;

pub mod migrations;

const LOG_TARGET: &str = "runtime::collective";

/// Simple index type for proposal counting.
pub type ProposalIndex = u32;

/// A number of members.
///
/// This also serves as a number of voting members, and since for motions, each member may
/// vote exactly once, therefore also the number of votes for any given motion.
pub type MemberCount = u32;

/// Default voting strategy when a member is inactive.
pub trait DefaultVote {
    /// Get the default voting strategy, given:
    ///
    /// - Whether the prime member voted Aye.
    /// - Raw number of yes votes.
    /// - Raw number of no votes.
    /// - Total number of member count.
    fn default_vote(
        prime_vote: Option<bool>,
        yes_votes: MemberCount,
        no_votes: MemberCount,
        len: MemberCount,
    ) -> bool;
}

/// Set the prime member's vote as the default vote.
pub struct PrimeDefaultVote;

impl DefaultVote for PrimeDefaultVote {
    fn default_vote(
        prime_vote: Option<bool>,
        _yes_votes: MemberCount,
        _no_votes: MemberCount,
        _len: MemberCount,
    ) -> bool {
        prime_vote.unwrap_or(false)
    }
}

/// First see if yes vote are over majority of the whole collective. If so, set the default vote
/// as yes. Otherwise, use the prime member's vote as the default vote.
pub struct MoreThanMajorityThenPrimeDefaultVote;

impl DefaultVote for MoreThanMajorityThenPrimeDefaultVote {
    fn default_vote(
        prime_vote: Option<bool>,
        yes_votes: MemberCount,
        _no_votes: MemberCount,
        len: MemberCount,
    ) -> bool {
        let more_than_majority = yes_votes * 2 > len;
        more_than_majority || prime_vote.unwrap_or(false)
    }
}

/// Origin for the collective module.
#[derive(PartialEq, Eq, Clone, RuntimeDebug, Encode, Decode, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(I))]
#[codec(mel_bound(AccountId: MaxEncodedLen))]
pub enum RawOrigin<AccountId, I> {
    /// It has been condoned by a given number of members of a collective from a given total.
    Members(MemberCount, MemberCount),
    /// It has been condoned by a single member of the collective.
    Member(AccountId),
    /// Dummy to manage the fact we have instancing.
    _Phantom(PhantomData<I>),
    /// It has been condened by a given number of groups in the council from a given total.
    Council(VotingGroupIndex, VotingGroupIndex),
}

impl<AccountId, I> GetBacking for RawOrigin<AccountId, I> {
    fn get_backing(&self) -> Option<Backing> {
        match self {
            RawOrigin::Members(n, d) => Some(Backing {
                approvals: *n,
                eligible: *d,
            }),
            RawOrigin::Council(n, d) => Some(Backing {
                approvals: *n,
                eligible: *d,
            }),
            _ => None,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode, RuntimeDebug, TypeInfo)]
pub enum VotingGroup {
    Senate,
    SubnetOwners,
    Triumvirate,
}

pub type VotingGroupIndex = u32;

pub type CollectiveThreshold = Permill;

/// Info for keeping track of a motion being voted on.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
pub struct Votes<AccountId, BlockNumber> {
    /// The proposal's unique index.
    index: ProposalIndex,
    /// The number of approval votes that are needed to pass the motion.
    threshold: CollectiveThreshold,
    /// The current set of voters that approved it.
    ayes: Vec<AccountId>,
    /// The current set of voters that rejected it.
    nays: Vec<AccountId>,
    /// The hard end time of this vote.
    end: BlockNumber,
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    /// The current storage version.
    const STORAGE_VERSION: StorageVersion = StorageVersion::new(4);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    #[pallet::without_storage_info]
    pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

    #[pallet::config]
    pub trait Config<I: 'static = ()>: frame_system::Config {
        /// The runtime origin type.
        type RuntimeOrigin: From<RawOrigin<Self::AccountId, I>>;

        /// The runtime call dispatch type.
        type Proposal: Parameter
            + Dispatchable<
                RuntimeOrigin = <Self as Config<I>>::RuntimeOrigin,
                PostInfo = PostDispatchInfo,
            > + From<frame_system::Call<Self>>
            + GetDispatchInfo;

        /// The runtime event type.
        type RuntimeEvent: From<Event<Self, I>>
            + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The time-out for council motions.
        type MotionDuration: Get<BlockNumberFor<Self>>;

        /// Maximum number of proposals allowed to be active in parallel.
        type MaxProposals: Get<ProposalIndex>;

        /// The maximum number of members supported by the pallet. Used for weight estimation.
        ///
        /// NOTE:
        /// + Benchmarks will need to be re-run and weights adjusted if this changes.
        /// + This pallet assumes that dependents keep to the limit without enforcing it.
        type MaxMembers: Get<MemberCount>;

        /// Default vote strategy of this collective.
        type DefaultVote: DefaultVote;

        /// Weight information for extrinsics in this pallet.
        type WeightInfo: WeightInfo;

        /// Origin allowed to set collective members
        type SetMembersOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;

        /// Origin allowed to propose
        type CanPropose: CanPropose<Self::AccountId>;

        /// Origin allowed to vote
        type CanVote: CanVote<Self::AccountId>;

        /// Members to expect in a vote
        type GetVotingMembers: GetVotingMembers<MemberCount>;

        /// The maximum number of voting groups supported by the pallet.
        ///
        /// This limits the number of groups that can be included in one
        /// collective vote.
        #[pallet::constant]
        type MaxVotingGroups: Get<VotingGroupIndex>;

        type CouncilGroups: Get<[VotingGroup; 2]>;

        type VoterThresholds: Get<[CollectiveThreshold; 2]>;
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
        pub phantom: PhantomData<I>,
        pub members: Vec<T::AccountId>,
    }

    impl<T: Config<I>, I: 'static> Default for GenesisConfig<T, I> {
        fn default() -> Self {
            Self {
                phantom: Default::default(),
                members: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config<I>, I: 'static> BuildGenesisConfig for GenesisConfig<T, I> {
        fn build(&self) {
            use sp_std::collections::btree_set::BTreeSet;
            let members_set: BTreeSet<_> = self.members.iter().collect();
            assert_eq!(
                members_set.len(),
                self.members.len(),
                "Members cannot contain duplicate accounts."
            );

            Pallet::<T, I>::initialize_members(&self.members)
        }
    }

    /// Origin for the collective pallet.
    #[pallet::origin]
    pub type Origin<T, I = ()> = RawOrigin<<T as frame_system::Config>::AccountId, I>;

    /// The hashes of the active proposals.
    #[pallet::storage]
    #[pallet::getter(fn proposals)]
    pub type Proposals<T: Config<I>, I: 'static = ()> =
        StorageValue<_, BoundedVec<T::Hash, T::MaxProposals>, ValueQuery>;

    /// Actual proposal for a given hash, if it's current.
    #[pallet::storage]
    #[pallet::getter(fn proposal_of)]
    pub type ProposalOf<T: Config<I>, I: 'static = ()> =
        StorageMap<_, Identity, T::Hash, <T as Config<I>>::Proposal, OptionQuery>;

    /// Votes on a given proposal, if it is ongoing. For all groups.
    #[pallet::storage] // --- DMAP ( proposal_hash, voting_group ) --> votes
    #[pallet::getter(fn voting)]
    pub type Voting<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
        _,
        Identity,
        T::Hash,
        Blake2_128Concat,
        VotingGroup,
        Votes<T::AccountId, BlockNumberFor<T>>,
        OptionQuery,
    >;

    /// Proposals so far.
    #[pallet::storage]
    #[pallet::getter(fn proposal_count)]
    pub type ProposalCount<T: Config<I>, I: 'static = ()> = StorageValue<_, u32, ValueQuery>;

    /// The current members of the collective. This is stored sorted (just by value).
    #[pallet::storage]
    #[pallet::getter(fn members)]
    pub type Members<T: Config<I>, I: 'static = ()> =
        StorageValue<_, Vec<T::AccountId>, ValueQuery>;

    /// The prime member that helps determine the default vote behavior in case of absentations.
    #[pallet::storage]
    #[pallet::getter(fn prime)]
    pub type Prime<T: Config<I>, I: 'static = ()> = StorageValue<_, T::AccountId, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config<I>, I: 'static = ()> {
        /// A motion (given hash) has been proposed (by given account) with a threshold (given
        /// `MemberCount`).
        Proposed {
            account: T::AccountId,
            proposal_index: ProposalIndex,
            proposal_hash: T::Hash,
            threshold: BoundedVec<CollectiveThreshold, T::MaxVotingGroups>,
        },
        /// A motion (given hash) has been voted on by given account, leaving
        /// a tally (yes votes and no votes given respectively as `MemberCount`).
        Voted {
            account: T::AccountId,
            proposal_hash: T::Hash,
            voted: bool,
            yes: MemberCount,
            no: MemberCount,
            voting_group: VotingGroup,
        },
        /// A motion was approved by the required threshold.
        Approved { proposal_hash: T::Hash },
        /// A motion was not approved by the required threshold.
        Disapproved { proposal_hash: T::Hash },
        /// A motion was executed; result will be `Ok` if it returned without error.
        Executed {
            proposal_hash: T::Hash,
            result: DispatchResult,
        },
        /// A single member did some action; result will be `Ok` if it returned without error.
        MemberExecuted {
            proposal_hash: T::Hash,
            result: DispatchResult,
        },
        /// A proposal was closed because its threshold was reached or after its duration was up.
        Closed {
            proposal_hash: T::Hash,
            yes: Vec<MemberCount>,
            no: Vec<MemberCount>,
        },
    }

    #[pallet::error]
    pub enum Error<T, I = ()> {
        /// Account is not a member
        NotMember,
        /// Duplicate proposals not allowed
        DuplicateProposal,
        /// Proposal must exist
        ProposalMissing,
        /// Mismatched index
        WrongIndex,
        /// Duplicate vote ignored
        DuplicateVote,
        /// Members are already initialized!
        AlreadyInitialized,
        /// The close call was made too early, before the end of the voting.
        TooEarly,
        /// There can only be a maximum of `MaxProposals` active proposals.
        TooManyProposals,
        /// The given weight bound for the proposal was too low.
        WrongProposalWeight,
        /// The given length bound for the proposal was too low.
        WrongProposalLength,
        /// The given motion duration for the proposal was too low.
        WrongDuration,
        /// The given voting group is not valid.
        UnknownVotingGroup,
        /// The origin is not privileged to propose
        NotPrivileged,
        /// The voting group has no members, or the members are not initialized
        EmptyVotingGroup,
        /// There is no prime member set, or the prime member did not vote
        NoPrimeMember,
        /// There is no early agreement on the proposal
        NoAgreement,
    }

    // Note that councillor operations are assigned to the operational class.
    #[pallet::call]
    impl<T: Config<I>, I: 'static> Pallet<T, I> {
        /// Set the collective's membership.
        ///
        /// - `new_members`: The new member list. Be nice to the chain and provide it sorted.
        /// - `prime`: The prime member whose vote sets the default.
        /// - `old_count`: The upper bound for the previous number of members in storage. Used for
        ///   weight estimation.
        ///
        /// The dispatch of this call must be `SetMembersOrigin`.
        ///
        /// NOTE: Does not enforce the expected `MaxMembers` limit on the amount of members, but
        ///       the weight estimations rely on it to estimate dispatchable weight.
        ///
        /// # WARNING:
        ///
        /// The `pallet-collective` can also be managed by logic outside of the pallet through the
        /// implementation of the trait [`ChangeMembers`].
        /// Any call to `set_members` must be careful that the member set doesn't get out of sync
        /// with other logic managing the member set.
        ///
        /// ## Complexity:
        /// - `O(MP + N)` where:
        ///   - `M` old-members-count (code- and governance-bounded)
        ///   - `N` new-members-count (code- and governance-bounded)
        ///   - `P` proposals-count (code-bounded)
        #[pallet::call_index(0)]
        #[pallet::weight((
			T::WeightInfo::set_members(
				*old_count, // M
				new_members.len() as u32, // N
				T::MaxProposals::get() // P
			),
			DispatchClass::Operational
		))]
        pub fn set_members(
            origin: OriginFor<T>,
            new_members: Vec<T::AccountId>,
            prime: Option<T::AccountId>,
            old_count: MemberCount,
        ) -> DispatchResultWithPostInfo {
            T::SetMembersOrigin::ensure_origin(origin)?;
            if new_members.len() > T::MaxMembers::get() as usize {
                log::error!(
                    target: LOG_TARGET,
                    "New members count ({}) exceeds maximum amount of members expected ({}).",
                    new_members.len(),
                    T::MaxMembers::get(),
                );
            }

            let old = Members::<T, I>::get();
            if old.len() > old_count as usize {
                log::warn!(
                    target: LOG_TARGET,
                    "Wrong count used to estimate set_members weight. expected ({}) vs actual ({})",
                    old_count,
                    old.len(),
                );
            }
            let mut new_members = new_members;
            new_members.sort();
            <Self as ChangeMembers<T::AccountId>>::set_members_sorted(&new_members, &old);
            Prime::<T, I>::set(prime);

            Ok(Some(T::WeightInfo::set_members(
                old.len() as u32,         // M
                new_members.len() as u32, // N
                T::MaxProposals::get(),   // P
            ))
            .into())
        }

        /// Add a new proposal to either be voted on or executed directly.
        ///
        /// Requires the sender to be member.
        ///
        /// `threshold` determines whether `proposal` is executed directly (`threshold < 2`)
        /// or put up for voting.
        ///
        /// ## Complexity
        /// - `O(B + M + P1)` or `O(B + M + P2)` where:
        ///   - `B` is `proposal` size in bytes (length-fee-bounded)
        ///   - `M` is members-count (code- and governance-bounded)
        ///   - branching is influenced by `threshold` where:
        ///     - `P1` is proposal execution complexity (`threshold < 2`)
        ///     - `P2` is proposals-count (code-bounded) (`threshold >= 2`)
        #[pallet::call_index(2)]
        #[pallet::weight((
			T::WeightInfo::propose_proposed(
				*length_bound, // B
				T::MaxMembers::get(), // M
				T::MaxProposals::get(), // P2
			),
			DispatchClass::Operational
		))]
        pub fn propose(
            origin: OriginFor<T>,
            proposal: Box<<T as Config<I>>::Proposal>,
            #[pallet::compact] length_bound: u32,
            duration: BlockNumberFor<T>,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin.clone())?;
            ensure!(
                T::CanPropose::can_propose(&who),
                Error::<T, I>::NotPrivileged
            );

            ensure!(
                duration >= T::MotionDuration::get(),
                Error::<T, I>::WrongDuration
            );

            let threshold = BoundedVec::truncate_from(T::VoterThresholds::get().to_vec());

            let (proposal_len, voting_members, active_proposals) =
                Self::do_propose_proposed(who, threshold, proposal, length_bound, duration)?;

            Ok(Some(T::WeightInfo::propose_proposed(
                proposal_len,     // B
                voting_members,   // M
                active_proposals, // P2
            ))
            .into())
        }

        /// Add an aye or nay vote for the sender to the given proposal.
        ///
        /// Requires the sender to be a member.
        ///
        /// Transaction fees will be waived if the member is voting on any particular proposal
        /// for the first time and the call is successful. Subsequent vote changes will charge a
        /// fee.
        /// ## Complexity
        /// - `O(M)` where `M` is members-count (code- and governance-bounded)
        #[pallet::call_index(3)]
        #[pallet::weight((T::WeightInfo::vote(T::MaxMembers::get()), DispatchClass::Operational))]
        pub fn vote(
            origin: OriginFor<T>,
            proposal: T::Hash,
            #[pallet::compact] index: ProposalIndex,
            approve: bool,
            group: VotingGroup,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin.clone())?;
            // Can vote as a member of the group
            ensure!(
                T::CanVote::can_vote_for_group(&who, group),
                Error::<T, I>::NotMember
            );
            // Ensure the group can vote
            ensure!(T::CanVote::can_vote(group), Error::<T, I>::NotPrivileged);

            let voter = if group == VotingGroup::Senate {
                // If Senate, grab the hotkey
                T::CanVote::get_member_account(&who, group) // Set the hotkey as the voter
            } else {
                who
            };

            // Get the members of the group
            let member_count = T::GetVotingMembers::get_count(group);

            // Detects first vote of the member in the motion
            let is_account_voting_first_time =
                Self::do_vote(voter, group, proposal, index, approve)?;

            if is_account_voting_first_time {
                Ok((Some(T::WeightInfo::vote(member_count)), Pays::No).into())
            } else {
                Ok((Some(T::WeightInfo::vote(member_count)), Pays::Yes).into())
            }
        }

        // NOTE: call_index(4) was `close_old_weight` and was removed due to weights v1
        // deprecation

        /// Disapprove a proposal, close, and remove it from the system, regardless of its current
        /// state.
        ///
        /// Must be called by the Root origin.
        ///
        /// Parameters:
        /// * `proposal_hash`: The hash of the proposal that should be disapproved.
        ///
        /// ## Complexity
        /// O(P) where P is the number of max proposals
        #[pallet::call_index(5)]
        #[pallet::weight(T::WeightInfo::disapprove_proposal(T::MaxProposals::get()))]
        pub fn disapprove_proposal(
            origin: OriginFor<T>,
            proposal_hash: T::Hash,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            let proposal_count = Self::do_disapprove_proposal(proposal_hash);
            Ok(Some(T::WeightInfo::disapprove_proposal(proposal_count)).into())
        }

        /// Close a vote that is either approved, disapproved or whose voting period has ended.
        ///
        /// May be called by any signed account in order to finish voting and close the proposal.
        ///
        /// If called before the end of the voting period it will only close the vote if it is
        /// has enough votes to be approved or disapproved.
        ///
        /// If called after the end of the voting period abstentions are counted as rejections
        /// unless there is a prime member set and the prime member cast an approval.
        ///
        /// If the close operation completes successfully with disapproval, the transaction fee will
        /// be waived. Otherwise execution of the approved operation will be charged to the caller.
        ///
        /// + `proposal_weight_bound`: The maximum amount of weight consumed by executing the closed
        /// proposal.
        /// + `length_bound`: The upper bound for the length of the proposal in storage. Checked via
        /// `storage::read` so it is `size_of::<u32>() == 4` larger than the pure length.
        ///
        /// ## Complexity
        /// - `O(B + M + P1 + P2)` where:
        ///   - `B` is `proposal` size in bytes (length-fee-bounded)
        ///   - `M` is members-count (code- and governance-bounded)
        ///   - `P1` is the complexity of `proposal` preimage.
        ///   - `P2` is proposal-count (code-bounded)
        #[pallet::call_index(6)]
        #[pallet::weight((
			{
				let b = *length_bound;
				let m = T::MaxMembers::get();
				let p1 = *proposal_weight_bound;
				let p2 = T::MaxProposals::get();
				T::WeightInfo::close_early_approved(b, m, p2)
					.max(T::WeightInfo::close_early_disapproved(m, p2))
					.max(T::WeightInfo::close_approved(b, m, p2))
					.max(T::WeightInfo::close_disapproved(m, p2))
					.saturating_add(p1)
			},
			DispatchClass::Operational
		))]
        pub fn close(
            origin: OriginFor<T>,
            proposal_hash: T::Hash,
            #[pallet::compact] index: ProposalIndex,
            proposal_weight_bound: Weight,
            #[pallet::compact] length_bound: u32,
        ) -> DispatchResultWithPostInfo {
            let _ = ensure_signed(origin)?;

            Self::do_close(proposal_hash, index, proposal_weight_bound, length_bound)
        }
    }
}

use frame_system::pallet_prelude::BlockNumberFor;

/// Return the weight of a dispatch call result as an `Option`.
///
/// Will return the weight regardless of what the state of the result is.
fn get_result_weight(result: DispatchResultWithPostInfo) -> Option<Weight> {
    match result {
        Ok(post_info) => post_info.actual_weight,
        Err(err) => err.post_info.actual_weight,
    }
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
    /// Check whether `who` is a member of the collective.
    pub fn is_member(who: &T::AccountId) -> bool {
        // Note: The dispatchables *do not* use this to check membership so make sure
        // to update those if this is changed.
        Self::members().contains(who)
    }

    /// Execute immediately when adding a new proposal.
    pub fn do_propose_execute(
        proposal: Box<<T as Config<I>>::Proposal>,
        length_bound: MemberCount,
    ) -> Result<(u32, DispatchResultWithPostInfo), DispatchError> {
        let proposal_len = proposal.encoded_size();
        ensure!(
            proposal_len <= length_bound as usize,
            Error::<T, I>::WrongProposalLength
        );

        let proposal_hash = T::Hashing::hash_of(&proposal);
        ensure!(
            !<ProposalOf<T, I>>::contains_key(proposal_hash),
            Error::<T, I>::DuplicateProposal
        );

        let seats = Self::members().len() as MemberCount;
        let result = proposal.dispatch(RawOrigin::Members(1, seats).into());
        Self::deposit_event(Event::Executed {
            proposal_hash,
            result: result.map(|_| ()).map_err(|e| e.error),
        });
        Ok((proposal_len as u32, result))
    }

    /// Add a new proposal to be voted.
    pub fn do_propose_proposed(
        who: T::AccountId,
        threshold: BoundedVec<CollectiveThreshold, T::MaxVotingGroups>,
        proposal: Box<<T as Config<I>>::Proposal>,
        length_bound: u32,
        duration: BlockNumberFor<T>,
    ) -> Result<(u32, u32, u32), DispatchError> {
        let proposal_len = proposal.encoded_size();
        ensure!(
            proposal_len <= length_bound as usize,
            Error::<T, I>::WrongProposalLength
        );

        let proposal_hash = T::Hashing::hash_of(&proposal);
        ensure!(
            !<ProposalOf<T, I>>::contains_key(proposal_hash),
            Error::<T, I>::DuplicateProposal
        );

        let active_proposals =
            <Proposals<T, I>>::try_mutate(|proposals| -> Result<usize, DispatchError> {
                proposals
                    .try_push(proposal_hash)
                    .map_err(|_| Error::<T, I>::TooManyProposals)?;
                Ok(proposals.len())
            })?;

        let index = Self::proposal_count();
        <ProposalCount<T, I>>::mutate(|i| *i += 1);
        <ProposalOf<T, I>>::insert(proposal_hash, proposal);

        let end = frame_system::Pallet::<T>::block_number() + duration;

        let mut voting_members: u32 = 0;

        // Insert new vote tracker for each voting group.
        let group_empty: Result<Vec<_>, Error<T, I>> = T::CouncilGroups::get()
            .iter()
            .enumerate()
            .map(|(i, group)| {
                let voting_group = *group;
                let member_count = T::GetVotingMembers::get_count(voting_group);
                voting_members += member_count;

                let votes = Votes {
                    index,
                    threshold: *threshold
                        .get(i)
                        .unwrap_or(&CollectiveThreshold::from_percent(50)), // Default to 50%
                    ayes: vec![],
                    nays: vec![],
                    end,
                };

                Voting::<T, I>::insert(proposal_hash, group, votes);

                Ok(())
            })
            .collect();
        ensure!(group_empty.is_ok(), group_empty.err().unwrap());

        Self::deposit_event(Event::Proposed {
            account: who,
            proposal_index: index,
            proposal_hash,
            threshold,
        });
        Ok((proposal_len as u32, voting_members, active_proposals as u32))
    }

    pub fn group_is_in_council(group: VotingGroup) -> bool {
        T::CouncilGroups::get().contains(&group)
    }

    /// Add an aye or nay vote for the member to the given proposal, returns true if it's the first
    /// vote of the member in the motion
    pub fn do_vote(
        who: T::AccountId,
        group: VotingGroup,
        proposal: T::Hash,
        index: ProposalIndex,
        approve: bool,
    ) -> Result<bool, DispatchError> {
        let mut voting: Votes<T::AccountId, BlockNumberFor<T>>;
        match group {
            VotingGroup::Senate => {
                voting =
                    Voting::<T, I>::get(proposal, group).ok_or(Error::<T, I>::ProposalMissing)?;
            }
            VotingGroup::SubnetOwners => {
                voting =
                    Voting::<T, I>::get(proposal, group).ok_or(Error::<T, I>::ProposalMissing)?;
            }
            _ => return Err(Error::<T, I>::UnknownVotingGroup.into()),
        }

        ensure!(voting.index == index, Error::<T, I>::WrongIndex);

        let position_yes = voting.ayes.iter().position(|a| a == &who);
        let position_no = voting.nays.iter().position(|a| a == &who);

        // Detects first vote of the member in the motion
        let is_account_voting_first_time = position_yes.is_none() && position_no.is_none();

        if approve {
            if position_yes.is_none() {
                voting.ayes.push(who.clone());
            } else {
                return Err(Error::<T, I>::DuplicateVote.into());
            }
            if let Some(pos) = position_no {
                voting.nays.swap_remove(pos);
            }
        } else {
            if position_no.is_none() {
                voting.nays.push(who.clone());
            } else {
                return Err(Error::<T, I>::DuplicateVote.into());
            }
            if let Some(pos) = position_yes {
                voting.ayes.swap_remove(pos);
            }
        }

        let yes_votes = voting.ayes.len() as MemberCount;
        let no_votes = voting.nays.len() as MemberCount;
        Self::deposit_event(Event::Voted {
            account: who,
            proposal_hash: proposal,
            voted: approve,
            yes: yes_votes,
            no: no_votes,
            voting_group: group,
        });

        Voting::<T, I>::insert(proposal, group, voting);

        Ok(is_account_voting_first_time)
    }

    fn passes_threshold(votes: MemberCount, seats: MemberCount, threshold: Permill) -> bool {
        // Return if the number of votes
        // surpasses the permill threshold of seats
        votes * 1_000_000 >= threshold * seats
    }

    fn check_votes(
        proposal_hash: T::Hash,
        index: ProposalIndex,
        group: VotingGroup,
    ) -> Result<
        (
            Votes<T::AccountId, BlockNumberFor<T>>,
            bool,
            bool,
            MemberCount,
            MemberCount,
            MemberCount,
        ),
        Error<T, I>,
    > {
        let group_voting =
            Self::voting(proposal_hash, group).ok_or(Error::<T, I>::ProposalMissing)?;
        ensure!(group_voting.index == index, Error::<T, I>::WrongIndex);

        let no_votes = group_voting.nays.len() as MemberCount;
        let yes_votes = group_voting.ayes.len() as MemberCount;

        let seats = T::GetVotingMembers::get_count(group);

        let approved = Self::passes_threshold(yes_votes, seats, group_voting.threshold);
        let disapproved = Self::passes_threshold(no_votes, seats, group_voting.threshold);

        Ok((
            group_voting,
            approved,
            disapproved,
            yes_votes,
            no_votes,
            seats,
        ))
    }

    /// Get the default vote for a given group.
    /// This function is used to determine the default vote for a group when a member abstains.
    fn get_default_vote(
        _: VotingGroup,
        group_voting: Votes<T::AccountId, BlockNumberFor<T>>,
        yes_votes: MemberCount,
        no_votes: MemberCount,
        seats: MemberCount,
    ) -> bool {
        // All groups use the same strategy.

        let prime_vote: Option<bool> = match Self::prime() {
            Some(who) => {
                if group_voting.ayes.iter().any(|a| a == &who) {
                    Some(true)
                } else if group_voting.nays.iter().any(|a| a == &who) {
                    Some(false)
                } else {
                    None // Prime member did not vote
                }
            }
            None => None, // No Prime member
        };

        // get the default voting strategy.
        T::DefaultVote::default_vote(prime_vote, yes_votes, no_votes, seats)
    }

    /// Close a vote that is either approved, disapproved or whose voting period has ended.
    pub fn do_close(
        proposal_hash: T::Hash,
        index: ProposalIndex,
        proposal_weight_bound: Weight,
        length_bound: u32,
    ) -> DispatchResultWithPostInfo {
        let council_groups = T::CouncilGroups::get();
        ensure!(!council_groups.is_empty(), Error::<T, I>::EmptyVotingGroup);
        ensure!(
            council_groups.len() <= T::MaxVotingGroups::get() as usize,
            Error::<T, I>::TooManyProposals
        );

        // Allow (dis-)approving the proposal as soon as there are enough votes.
        let mut decided_early_results: BoundedVec<
            // This format allows unzip later.
            // decision, (yes_votes, (no_votes, seats))
            (bool, (MemberCount, (MemberCount, MemberCount))),
            T::MaxVotingGroups,
        > = BoundedVec::new();

        let decided_early: Result<_, Error<T, I>> = council_groups.iter().try_for_each(|group| {
            let (_group_voting, approved, disapproved, yes_votes, no_votes, seats) =
                Self::check_votes(proposal_hash, index, *group)?;

            if approved ^ disapproved {
                let decision = !disapproved; // True for approval, false for disapproval

                if decided_early_results
                    .last()
                    .unwrap_or(&(false, (0, (0, 0))))
                    .0
                    != decision
                {
                    // Different results, not in agreement with the previous group(s)
                    return Err(Error::<T, I>::NoAgreement);
                } // Otherwise, continue
                decided_early_results
                    .try_push((decision, (yes_votes, (no_votes, seats))))
                    .defensive_map_err(|_|
					// This should not happen, as we checked the length before
					Error::<T, I>::TooManyProposals) // Continue
            } else {
                Err(Error::<T, I>::TooEarly) // Not decided yet
            }
        });

        if decided_early.is_ok() {
            // Threshold is passed and agreed for all groups

            // Verify the length of results. Should not be different.
            ensure!(
                decided_early_results.len() == council_groups.len(),
                Error::<T, I>::NoAgreement
            );

            let (approved_group, (yes_votes, (no_votes, seats))): (
                Vec<bool>,
                (Vec<MemberCount>, (Vec<MemberCount>, Vec<MemberCount>)),
            ) = decided_early_results.into_iter().unzip();
            // All groups have the same decision
            let approved: bool = *approved_group.first().defensive_unwrap_or(&false);

            let seats_total: u32 = seats.iter().sum();

            if approved {
                let (proposal, len) = Self::validate_and_get_proposal(
                    &proposal_hash,
                    length_bound,
                    proposal_weight_bound,
                )?;

                Self::deposit_event(Event::Closed {
                    proposal_hash,
                    yes: yes_votes,
                    no: no_votes,
                });

                let (proposal_weight, proposal_count) = Self::do_approve_proposal(
                    approved_group.iter().filter(|&a| *a).count() as VotingGroupIndex,
                    council_groups.len() as VotingGroupIndex,
                    proposal_hash,
                    proposal,
                );

                return Ok((
                    Some(
                        T::WeightInfo::close_early_approved(
                            len as u32,
                            seats_total,
                            proposal_count,
                        )
                        .saturating_add(proposal_weight),
                    ),
                    Pays::Yes,
                )
                    .into());
            } else {
                // Disapproved
                Self::deposit_event(Event::Closed {
                    proposal_hash,
                    yes: yes_votes,
                    no: no_votes,
                });
                let proposal_count = Self::do_disapprove_proposal(proposal_hash);
                return Ok((
                    Some(T::WeightInfo::close_early_disapproved(
                        seats_total,
                        proposal_count,
                    )),
                    Pays::No,
                )
                    .into());
            }
        }

        // If the proposal was not decided early, check if the voting period has ended
        #[allow(clippy::single_match)] // May become more complex in the future
        match decided_early {
            Err(Error::<T, I>::TooEarly) => decided_early?,
            _ => (),
        }
        // No early approval or disapproval, check if the voting period has ended
        let (first_group_voting, _approved, _disapproved, _yes_votes, _no_votes, _seats) =
            Self::check_votes(
                proposal_hash,
                index,
                *council_groups
                    .first()
                    .defensive_unwrap_or(&VotingGroup::Senate),
            )?;

        // Only allow final closing of the proposal after the voting period has ended.
        ensure!(
            frame_system::Pallet::<T>::block_number() >= first_group_voting.end,
            Error::<T, I>::TooEarly
        );

        // Iterate over all remaining groups (if any)
        let last_index = decided_early_results.len() - 1;
        let mut all_results = decided_early_results;
        if council_groups.len() - 1 > last_index {
            // Some groups were not tallied yet
            let remaining_groups = &council_groups[last_index + 1..];
            for group in remaining_groups.iter() {
                let (group_voting, approved, disapproved, mut yes_votes, mut no_votes, seats) =
                    Self::check_votes(proposal_hash, index, *group)?;

                if approved || disapproved {
                    all_results
                        .try_push((approved, (yes_votes, (no_votes, seats))))
                        .defensive_map_err(|_| Error::<T, I>::TooManyProposals)?;
                // Should not happen
                } else {
                    let group_threshold = group_voting.threshold;
                    // Not decided yet, use the default vote for all abstentions in the group
                    let default_vote: bool =
                        Self::get_default_vote(*group, group_voting, yes_votes, no_votes, seats);

                    let abstentions = seats - (yes_votes + no_votes);
                    match default_vote {
                        true => yes_votes += abstentions,
                        false => no_votes += abstentions,
                    }

                    // Check approval using the default vote; Disapproved if not approved
                    let approved = Self::passes_threshold(yes_votes, seats, group_threshold);

                    all_results
                        .try_push((approved, (yes_votes, (no_votes, seats))))
                        .defensive_map_err(|_| Error::<T, I>::TooManyProposals)?;
                    // Should not happen
                }
            }
        }

        // Check if the proposal is approved or disapproved
        let (approved_group, (yes_votes, (no_votes, seats))): (
            Vec<bool>,
            (Vec<MemberCount>, (Vec<MemberCount>, Vec<MemberCount>)),
        ) = all_results.into_iter().unzip();
        let approved = approved_group.iter().all(|&a| a);

        let seats_total: u32 = seats.iter().sum();

        if approved {
            let (proposal, len) = Self::validate_and_get_proposal(
                &proposal_hash,
                length_bound,
                proposal_weight_bound,
            )?;
            Self::deposit_event(Event::Closed {
                proposal_hash,
                yes: yes_votes,
                no: no_votes,
            });
            let (proposal_weight, proposal_count) = Self::do_approve_proposal(
                approved_group.iter().filter(|&a| *a).count() as VotingGroupIndex,
                council_groups.len() as VotingGroupIndex,
                proposal_hash,
                proposal,
            );

            Ok((
                Some(
                    T::WeightInfo::close_approved(len as u32, seats_total, proposal_count)
                        .saturating_add(proposal_weight),
                ),
                Pays::Yes,
            )
                .into())
        } else {
            // Disapproved by at least one group

            Self::deposit_event(Event::Closed {
                proposal_hash,
                yes: yes_votes,
                no: no_votes,
            });
            let proposal_count = Self::do_disapprove_proposal(proposal_hash);

            Ok((
                Some(T::WeightInfo::close_disapproved(
                    seats_total,
                    proposal_count,
                )),
                Pays::No,
            )
                .into())
        }
    }

    /// Ensure that the right proposal bounds were passed and get the proposal from storage.
    ///
    /// Checks the length in storage via `storage::read` which adds an extra `size_of::<u32>() == 4`
    /// to the length.
    fn validate_and_get_proposal(
        hash: &T::Hash,
        length_bound: u32,
        weight_bound: Weight,
    ) -> Result<(<T as Config<I>>::Proposal, usize), DispatchError> {
        let key = ProposalOf::<T, I>::hashed_key_for(hash);
        // read the length of the proposal storage entry directly
        let proposal_len =
            storage::read(&key, &mut [0; 0], 0).ok_or(Error::<T, I>::ProposalMissing)?;
        ensure!(
            proposal_len <= length_bound,
            Error::<T, I>::WrongProposalLength
        );
        let proposal = ProposalOf::<T, I>::get(hash).ok_or(Error::<T, I>::ProposalMissing)?;
        let proposal_weight = proposal.get_dispatch_info().weight;
        ensure!(
            proposal_weight.all_lte(weight_bound),
            Error::<T, I>::WrongProposalWeight
        );
        Ok((proposal, proposal_len as usize))
    }

    /// Weight:
    /// If `approved`:
    /// - the weight of `proposal` preimage.
    /// - two events deposited.
    /// - two removals, one mutation.
    /// - computation and i/o `O(P + L)` where:
    ///   - `P` is number of active proposals,
    ///   - `L` is the encoded length of `proposal` preimage.
    ///
    /// If not `approved`:
    /// - one event deposited.
    /// Two removals, one mutation.
    /// Computation and i/o `O(P)` where:
    /// - `P` is number of active proposals
    fn do_approve_proposal(
        yes_votes: VotingGroupIndex,
        groups: VotingGroupIndex,
        proposal_hash: T::Hash,
        proposal: <T as Config<I>>::Proposal,
    ) -> (Weight, u32) {
        Self::deposit_event(Event::Approved { proposal_hash });

        let dispatch_weight = proposal.get_dispatch_info().weight;

        // Number of approving groups out of the total groups
        let origin = RawOrigin::Council(yes_votes, groups).into();
        let result = proposal.dispatch(origin);
        Self::deposit_event(Event::Executed {
            proposal_hash,
            result: result.map(|_| ()).map_err(|e| e.error),
        });
        // default to the dispatch info weight for safety
        let proposal_weight = get_result_weight(result).unwrap_or(dispatch_weight); // P1

        let proposal_count = Self::remove_proposal(proposal_hash);
        (proposal_weight, proposal_count)
    }

    /// Removes a proposal from the pallet, and deposit the `Disapproved` event.
    pub fn do_disapprove_proposal(proposal_hash: T::Hash) -> u32 {
        // disapproved
        Self::deposit_event(Event::Disapproved { proposal_hash });
        Self::remove_proposal(proposal_hash)
    }

    // Removes a proposal from the pallet, cleaning up votes and the vector of proposals.
    fn remove_proposal(proposal_hash: T::Hash) -> u32 {
        // remove proposal and votes involving proposal
        ProposalOf::<T, I>::remove(proposal_hash);
        let _ = Voting::<T, I>::clear_prefix(proposal_hash, T::MaxVotingGroups::get(), None);

        let num_proposals = Proposals::<T, I>::mutate(|proposals| {
            proposals.retain(|h| h != &proposal_hash);
            proposals.len() + 1 // calculate weight based on original length
        });
        num_proposals as u32
    }

    pub fn remove_votes(who: &T::AccountId, group: VotingGroup) -> Result<bool, DispatchError> {
        for h in Self::proposals().into_iter() {
            <Voting<T, I>>::mutate(h, group, |v| {
                if let Some(mut votes) = v.take() {
                    votes.ayes.retain(|i| i != who);
                    votes.nays.retain(|i| i != who);
                    *v = Some(votes);
                }
            });
        }

        Ok(true)
    }

    pub fn has_voted(
        proposal: T::Hash,
        index: ProposalIndex,
        who: &T::AccountId,
        group: VotingGroup,
    ) -> Result<bool, DispatchError> {
        let voting;
        let who = who.clone();
        match group {
            VotingGroup::Senate => {
                voting = Self::voting(proposal, group).ok_or(Error::<T, I>::ProposalMissing)?;
            }
            VotingGroup::SubnetOwners => {
                voting = Self::voting(proposal, group).ok_or(Error::<T, I>::ProposalMissing)?;
            }
            _ => return Err(Error::<T, I>::UnknownVotingGroup.into()),
        }

        ensure!(voting.index == index, Error::<T, I>::WrongIndex);

        let position_yes = voting.ayes.iter().position(|a| *a == who);
        let position_no = voting.nays.iter().position(|a| *a == who);

        Ok(position_yes.is_some() || position_no.is_some())
    }
}

impl<T: Config<I>, I: 'static> ChangeMembers<T::AccountId> for Pallet<T, I> {
    /// Update the members of the collective. Votes are updated and the prime is reset.
    ///
    /// NOTE: Does not enforce the expected `MaxMembers` limit on the amount of members, but
    ///       the weight estimations rely on it to estimate dispatchable weight.
    ///
    /// ## Complexity
    /// - `O(MP + N)`
    ///   - where `M` old-members-count (governance-bounded)
    ///   - where `N` new-members-count (governance-bounded)
    ///   - where `P` proposals-count
    fn change_members_sorted(
        _incoming: &[T::AccountId],
        outgoing: &[T::AccountId],
        new: &[T::AccountId],
    ) {
        if new.len() > T::MaxMembers::get() as usize {
            log::error!(
                target: LOG_TARGET,
                "New members count ({}) exceeds maximum amount of members expected ({}).",
                new.len(),
                T::MaxMembers::get(),
            );
        }
        // remove accounts from all current voting in motions.
        let mut outgoing = outgoing.to_vec();
        outgoing.sort();
        let council_groups = T::CouncilGroups::get();

        for h in Self::proposals().into_iter() {
            // Iterate over all groups that can vote
            for group in council_groups.iter() {
                <Voting<T, I>>::mutate(h, group, |v| {
                    if let Some(mut votes) = v.take() {
                        votes.ayes.retain(|i| outgoing.binary_search(i).is_err());
                        votes.nays.retain(|i| outgoing.binary_search(i).is_err());
                        *v = Some(votes);
                    }
                });
            }
        }
        Members::<T, I>::put(new);
        Prime::<T, I>::kill();
    }

    fn set_prime(prime: Option<T::AccountId>) {
        Prime::<T, I>::set(prime);
    }

    fn get_prime() -> Option<T::AccountId> {
        Prime::<T, I>::get()
    }
}

impl<T: Config<I>, I: 'static> InitializeMembers<T::AccountId> for Pallet<T, I> {
    fn initialize_members(members: &[T::AccountId]) {
        if !members.is_empty() {
            assert!(
                <Members<T, I>>::get().is_empty(),
                "Members are already initialized!"
            );
            <Members<T, I>>::put(members);
        }
    }
}

/// Ensure that the origin `o` represents at least `n` members. Returns `Ok` or an `Err`
/// otherwise.
pub fn ensure_members<OuterOrigin, AccountId, I>(
    o: OuterOrigin,
    n: MemberCount,
) -> result::Result<MemberCount, &'static str>
where
    OuterOrigin: Into<result::Result<RawOrigin<AccountId, I>, OuterOrigin>>,
{
    match o.into() {
        Ok(RawOrigin::Members(x, _)) if x >= n => Ok(n),
        _ => Err("bad origin: expected to be a threshold number of members"),
    }
}

pub struct EnsureMember<AccountId, I: 'static>(PhantomData<(AccountId, I)>);
impl<
        O: Into<Result<RawOrigin<AccountId, I>, O>> + From<RawOrigin<AccountId, I>>,
        I,
        AccountId: Decode,
    > EnsureOrigin<O> for EnsureMember<AccountId, I>
{
    type Success = AccountId;
    fn try_origin(o: O) -> Result<Self::Success, O> {
        o.into().and_then(|o| match o {
            RawOrigin::Member(id) => Ok(id),
            r => Err(O::from(r)),
        })
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn try_successful_origin() -> Result<O, ()> {
        let zero_account_id =
            AccountId::decode(&mut sp_runtime::traits::TrailingZeroInput::zeroes())
                .expect("infinite length input; no invalid inputs for type; qed");
        Ok(O::from(RawOrigin::Member(zero_account_id)))
    }
}

pub struct EnsureMembers<AccountId, I: 'static, const N: u32>(PhantomData<(AccountId, I)>);
impl<
        O: Into<Result<RawOrigin<AccountId, I>, O>> + From<RawOrigin<AccountId, I>>,
        AccountId,
        I,
        const N: u32,
    > EnsureOrigin<O> for EnsureMembers<AccountId, I, N>
{
    type Success = (MemberCount, MemberCount);
    fn try_origin(o: O) -> Result<Self::Success, O> {
        o.into().and_then(|o| match o {
            RawOrigin::Members(n, m) if n >= N => Ok((n, m)),
            r => Err(O::from(r)),
        })
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn try_successful_origin() -> Result<O, ()> {
        Ok(O::from(RawOrigin::Members(N, N)))
    }
}

pub struct EnsureProportionMoreThan<AccountId, I: 'static, const N: u32, const D: u32>(
    PhantomData<(AccountId, I)>,
);
impl<
        O: Into<Result<RawOrigin<AccountId, I>, O>> + From<RawOrigin<AccountId, I>>,
        AccountId,
        I,
        const N: u32,
        const D: u32,
    > EnsureOrigin<O> for EnsureProportionMoreThan<AccountId, I, N, D>
{
    type Success = ();
    fn try_origin(o: O) -> Result<Self::Success, O> {
        o.into().and_then(|o| match o {
            RawOrigin::Members(n, m) if n * D > N * m => Ok(()),
            r => Err(O::from(r)),
        })
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn try_successful_origin() -> Result<O, ()> {
        Ok(O::from(RawOrigin::Members(1u32, 0u32)))
    }
}

/// Variants of RawOrigin that are used by the EnsureOrigin implementations.
pub trait OriginVariant {}
pub struct MembersVariant;
pub struct CouncilVariant;

impl OriginVariant for MembersVariant {}
impl OriginVariant for CouncilVariant {}

pub struct EnsureUnanimous<AccountId, I: 'static, Variant>(PhantomData<(AccountId, I, Variant)>);

impl<O: Into<Result<RawOrigin<AccountId, I>, O>> + From<RawOrigin<AccountId, I>>, AccountId, I>
    EnsureOrigin<O> for EnsureUnanimous<AccountId, I, MembersVariant>
{
    type Success = ();
    fn try_origin(o: O) -> Result<Self::Success, O> {
        o.into().and_then(|o| match o {
            RawOrigin::Members(n, m) if n == m => Ok(()),
            _ => Err(O::from(o)),
        })
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn try_successful_origin() -> Result<O, ()> {
        Ok(O::from(RawOrigin::Members(0u32, 0u32)))
    }
}

impl<O: Into<Result<RawOrigin<AccountId, I>, O>> + From<RawOrigin<AccountId, I>>, AccountId, I>
    EnsureOrigin<O> for EnsureUnanimous<AccountId, I, CouncilVariant>
{
    type Success = ();
    fn try_origin(o: O) -> Result<Self::Success, O> {
        o.into().and_then(|o| match o {
            RawOrigin::Council(n, m) if n == m => Ok(()),
            _ => Err(O::from(o)),
        })
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn try_successful_origin() -> Result<O, ()> {
        Ok(O::from(RawOrigin::Council(0u32, 0u32)))
    }
}

pub struct EnsureProportionAtLeast<AccountId, I: 'static, const N: u32, const D: u32>(
    PhantomData<(AccountId, I)>,
);
impl<
        O: Into<Result<RawOrigin<AccountId, I>, O>> + From<RawOrigin<AccountId, I>>,
        AccountId,
        I,
        const N: u32,
        const D: u32,
    > EnsureOrigin<O> for EnsureProportionAtLeast<AccountId, I, N, D>
{
    type Success = ();
    fn try_origin(o: O) -> Result<Self::Success, O> {
        o.into().and_then(|o| match o {
            RawOrigin::Members(n, m) if n * D >= N * m => Ok(()),
            r => Err(O::from(r)),
        })
    }

    #[cfg(feature = "runtime-benchmarks")]
    fn try_successful_origin() -> Result<O, ()> {
        Ok(O::from(RawOrigin::Members(0u32, 0u32)))
    }
}

/// CanPropose
pub trait CanPropose<AccountId> {
    /// Check whether or not the passed AccountId can propose a new motion
    fn can_propose(account: &AccountId) -> bool;
}

impl<T> CanPropose<T> for () {
    fn can_propose(_: &T) -> bool {
        false
    }
}

/// CanVote
pub trait CanVote<AccountId> {
    /// Check whether or not the passed AccountId can vote on a motion
    /// given they're voting as a certain group.
    fn can_vote_for_group(account: &AccountId, group: VotingGroup) -> bool;

    fn can_vote(group: VotingGroup) -> bool;

    fn get_member_account(account: &AccountId, group: VotingGroup) -> AccountId;
}

impl<T> CanVote<T> for ()
where
    T: Clone,
{
    fn can_vote_for_group(_: &T, _: VotingGroup) -> bool {
        false
    }

    fn can_vote(_: VotingGroup) -> bool {
        false
    }

    fn get_member_account(a: &T, _: VotingGroup) -> T {
        a.clone()
    }
}

pub trait GetVotingMembers<MemberCount> {
    fn get_count(group: VotingGroup) -> MemberCount;
}

impl GetVotingMembers<MemberCount> for () {
    fn get_count(_: VotingGroup) -> MemberCount {
        0
    }
}
