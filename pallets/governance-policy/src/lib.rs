#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{boxed::Box, vec, vec::Vec};
use frame_support::{
    dispatch::{GetDispatchInfo, RawOrigin},
    pallet_prelude::*,
    sp_runtime::{
        Saturating,
        traits::{BlockNumberProvider, Dispatchable, Hash, UniqueSaturatedInto},
    },
    traits::{IsSubType, StorePreimage, schedule::Priority},
};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use pallet_multi_collective::OnNewTerm;
use pallet_referenda::{Proposal, TracksInfo};
use substrate_fixed::types::I96F32;

pub type CallOf<T> = <T as Config>::RuntimeCall;

pub type TrackIdOf<T> = <<T as pallet_referenda::Config>::Tracks as TracksInfo<
    pallet_referenda::TrackName,
    <T as frame_system::Config>::AccountId,
    pallet_referenda::CallOf<T>,
    BlockNumberFor<T>,
>>::Id;

#[frame_support::pallet]
#[allow(clippy::expect_used)]
pub mod pallet {
    use super::*;

    const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

    #[pallet::pallet]
    #[pallet::storage_version(STORAGE_VERSION)]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config:
        frame_system::Config
        + pallet_referenda::Config
        + pallet_scheduler::Config<
            BlockNumberProvider: BlockNumberProvider<BlockNumber = BlockNumberFor<Self>>,
        >
        + pallet_subtensor_utility::Config<
            PalletsOrigin: From<frame_system::RawOrigin<<Self as frame_system::Config>::AccountId>>,
        >
        + pallet_multi_collective::Config<CollectiveId: Eq>
        + pallet_subtensor::Config
    {
        /// The overarching call type. Must be convertible from the inner
        /// pallet `Call<Self>` types we batch together — that is what
        /// allows this pallet to *build* a runtime call without knowing
        /// the concrete `RuntimeCall` enum.
        type RuntimeCall: Parameter
            + Dispatchable<RuntimeOrigin = <Self as frame_system::Config>::RuntimeOrigin>
            + GetDispatchInfo
            + From<frame_system::Call<Self>>
            + From<pallet_referenda::Call<Self>>
            + From<pallet_scheduler::Call<Self>>
            + From<pallet_subtensor_utility::Call<Self>>
            + IsSubType<Call<Self>>
            + IsType<<Self as frame_system::Config>::RuntimeCall>
            + IsType<<Self as pallet_referenda::Config>::RuntimeCall>
            + IsType<<Self as pallet_scheduler::Config>::RuntimeCall>
            + IsType<<Self as pallet_subtensor_utility::Config>::RuntimeCall>;

        /// The track on which the triumvirate votes (approves/rejects the
        /// upgrade). The wrapped call lands here as an `Action`.
        #[pallet::constant]
        type TriumvirateTrack: Get<TrackIdOf<Self>>;

        /// The track on which the collectives review the *timing* of the
        /// approved upgrade. The Review referendum is created here.
        #[pallet::constant]
        type ReviewTrack: Get<TrackIdOf<Self>>;

        /// Initial delay between triumvirate approval and the eventual
        /// dispatch of the wrapped call. This is the window during which
        /// the collectives can fast-track / cancel / extend.
        #[pallet::constant]
        type InitialSchedulingDelay: Get<BlockNumberFor<Self>>;

        /// `CollectiveId` whose membership is "top validators by stake".
        #[pallet::constant]
        type EconomicCollective: Get<<Self as pallet_multi_collective::Config>::CollectiveId>;

        /// `CollectiveId` whose membership is "top subnet owners by
        /// moving-average price".
        #[pallet::constant]
        type BuildingCollective: Get<<Self as pallet_multi_collective::Config>::CollectiveId>;

        /// Target size of each ranked collective (DESIGN.md: 16 each).
        #[pallet::constant]
        type CollectiveSize: Get<u32>;

        /// Minimum subnet age (in blocks) for its owner to be eligible
        /// for the Building collective (DESIGN.md: 6 months).
        #[pallet::constant]
        type MinSubnetAge: Get<BlockNumberFor<Self>>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A new governance referendum has been created. The submitter is
        /// the proposer that called `create_referendum`; the task name is
        /// the deterministic id under which the wrapped call will be
        /// scheduled if the triumvirate approves.
        ReferendumCreated {
            who: T::AccountId,
            task_id: pallet_referenda::ProposalTaskName,
        },
        /// A ranked collective was repopulated. `kind` distinguishes
        /// Economic (validators) from Building (subnet owners). `members`
        /// is the new full membership list — the multi-collective pallet
        /// also emits `MembersReset`, but that event lacks the rotation-
        /// kind context.
        CollectiveRotated {
            collective_id: <T as pallet_multi_collective::Config>::CollectiveId,
            kind: RotationKind,
            members: Vec<T::AccountId>,
        },
        /// `OnNewTerm` fired for a collective that this pallet does not
        /// manage. Logged for observability — could happen if the runtime
        /// adds a new term-bound collective and forgets to extend the
        /// match arm here.
        UnmanagedCollectiveTerm {
            collective_id: <T as pallet_multi_collective::Config>::CollectiveId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// The user-provided call could not be hashed into a 32-byte task
        /// name. Should not happen with a `Hash = H256` runtime, but we
        /// surface it explicitly rather than panic.
        InvalidTaskName,
        /// `force_rotate` was called for a collective id that is neither
        /// `EconomicCollective` nor `BuildingCollective`.
        UnmanagedCollective,
    }

    /// Tag attached to `CollectiveRotated` so external observers can tell
    /// which side of the policy fired without re-reading config.
    #[derive(
        Encode, Decode, DecodeWithMemTracking, MaxEncodedLen, Clone, Copy,
        PartialEq, Eq, TypeInfo, Debug,
    )]
    pub enum RotationKind {
        Economic,
        Building,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create the two-phase governance referendum for `call`.
        ///
        /// Origin: signed. Authorisation is delegated to
        /// `pallet-referenda` via the `TriumvirateTrack`'s proposer set —
        /// callers that are not in the proposer set will be rejected by
        /// `referenda.submit` with `NotProposer`.
        #[pallet::call_index(0)]
        #[pallet::weight({
            let info = call.get_dispatch_info();
            (
                info.call_weight.saturating_add(T::DbWeight::get().reads_writes(4, 4)),
                info.class,
            )
        })]
        pub fn create_referendum(
            origin: OriginFor<T>,
            call: Box<CallOf<T>>,
        ) -> DispatchResult {
            // We *do not* gate the origin here. `pallet-referenda` checks
            // the proposer set via `track.proposer_set.contains(...)` when
            // the Action proposal is submitted below.
            let who = ensure_signed(origin.clone())?;

            let (action_call, task_id) = Self::build_governance_action(*call)?;
            let referenda_call: <T as pallet_referenda::Config>::RuntimeCall = action_call.into();
            let bounded =
                <T as pallet_referenda::Config>::Preimages::bound(referenda_call)
                    .map_err(|_| Error::<T>::InvalidTaskName)?;


            pallet_referenda::Pallet::<T>::submit(
                origin,
                T::TriumvirateTrack::get(),
                Proposal::Action(bounded),
            )?;

            Self::deposit_event(Event::<T>::ReferendumCreated { who, task_id });
            Ok(())
        }

        /// Manually rotate the membership of a ranked collective. Used
        /// for the very first population (genesis collectives are empty
        /// and the natural rotation only fires at the first term boundary,
        /// which is up to 60 days in) and as a Root override during
        /// incidents.
        ///
        /// Origin: Root. Restricted to the Economic and Building
        /// collectives — proposers and triumvirate are managed by Root
        /// directly via `pallet-multi-collective::reset_members`.
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(500_000_000, 0)
            .saturating_add(T::DbWeight::get().reads_writes(8, 4)))]
        pub fn force_rotate(
            origin: OriginFor<T>,
            collective_id: <T as pallet_multi_collective::Config>::CollectiveId,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;

            if collective_id == T::EconomicCollective::get() {
                let weight = Self::rotate_economic();
                Ok(Some(weight).into())
            } else if collective_id == T::BuildingCollective::get() {
                let weight = Self::rotate_building();
                Ok(Some(weight).into())
            } else {
                Err(Error::<T>::UnmanagedCollective.into())
            }
        }
    }
}

/// `OnNewTerm` is invoked by `pallet-multi-collective::on_initialize`
/// when a collective's `term_duration` divides the current block height.
/// We dispatch by collective id to the matching ranking pass, returning
/// the actual storage cost so multi-collective can bill the block.
impl<T: Config> OnNewTerm<<T as pallet_multi_collective::Config>::CollectiveId> for Pallet<T> {
    fn on_new_term(
        collective_id: <T as pallet_multi_collective::Config>::CollectiveId,
    ) -> Weight {
        if collective_id == T::EconomicCollective::get() {
            Self::rotate_economic()
        } else if collective_id == T::BuildingCollective::get() {
            Self::rotate_building()
        } else {
            // Triumvirate / Proposers etc. don't auto-rotate. Emit so a
            // misconfiguration (term_duration set on a non-ranked
            // collective) is observable instead of silent.
            Self::deposit_event(Event::<T>::UnmanagedCollectiveTerm { collective_id });
            T::DbWeight::get().reads(0)
        }
    }
}

impl<T: Config> Pallet<T> {
    /// Build the batched Action call that the triumvirate will vote on.
    ///
    /// The shape mirrors `DESIGN.md`'s worked example, with the addition
    /// of a `dispatch_as(Root, ...)` wrap around `inner_call` (see crate
    /// docs for the rationale).
    fn build_governance_action(
        inner_call: CallOf<T>,
    ) -> Result<(CallOf<T>, pallet_referenda::ProposalTaskName), DispatchError> {
        let now = <frame_system::Pallet<T>>::block_number();
        let when = now.saturating_add(T::InitialSchedulingDelay::get());

        // Step 1: pin the inner-call origin to Root via `dispatch_as`.
        // Even when the batch_all that contains the schedule_named call
        // already runs as Root, this wrap forces the *deferred* dispatch
        // to run as Root regardless of how the scheduler captures /
        // re-casts the calling origin in `do_schedule_named`.
        let dispatch_as_call: CallOf<T> = pallet_subtensor_utility::Call::<T>::dispatch_as {
            as_origin: Box::new(
                frame_system::RawOrigin::<T::AccountId>::Root.into(),
            ),
            call: Box::new(inner_call.into()),
        }
        .into();

        // Step 2: derive a deterministic but *unique-per-submission*
        // task name. We mix the next referendum index into the hash so
        // that two proposals carrying the same call do not collide on
        // `schedule_named`'s name registry — the v1 pallet sidestepped
        // this by rejecting duplicate calls outright, but here we want
        // to allow re-submission (e.g. the same patch through different
        // tracks) without DoS'ing the second one.
        let next_index = pallet_referenda::ReferendumCount::<T>::get();
        let task_id = Self::derive_task_id(&dispatch_as_call, next_index)?;

        // Step 3: schedule_named(task_id, when, dispatch_as(Root, call))
        let schedule_call: CallOf<T> = pallet_scheduler::Call::<T>::schedule_named {
            id: task_id,
            when,
            maybe_periodic: None,
            priority: Priority::default(),
            call: Box::new(dispatch_as_call.into()),
        }
        .into();

        // Step 4: referenda.submit(ReviewTrack, Review(task_id))
        let review_submit: CallOf<T> = pallet_referenda::Call::<T>::submit {
            track: T::ReviewTrack::get(),
            proposal: Proposal::<pallet_referenda::BoundedCallOf<T>>::Review(task_id),
        }
        .into();

        // Step 5: batch_all([schedule_call, review_submit])
        let batch_call: CallOf<T> = pallet_subtensor_utility::Call::<T>::batch_all {
            calls: vec![schedule_call.into(), review_submit.into()],
        }
        .into();

        Ok((batch_call, task_id))
    }

    /// Derive the 32-byte task name from a constructed call mixed with
    /// the next referendum index. The index is the per-pallet primary
    /// key in `pallet-referenda`; tying the task name to it gives us
    /// uniqueness without needing extra storage here.
    fn derive_task_id(
        call: &CallOf<T>,
        index: pallet_referenda::ReferendumIndex,
    ) -> Result<pallet_referenda::ProposalTaskName, DispatchError> {
        let hash =
            <T as frame_system::Config>::Hashing::hash_of(&(b"governance-policy", call, index));
        hash.as_ref()
            .try_into()
            .map_err(|_| Error::<T>::InvalidTaskName.into())
    }

    fn rotate_economic() -> Weight {
        let target = T::CollectiveSize::get();
        let (members, query_weight) = Self::top_validators(target);
        Self::apply_rotation(
            T::EconomicCollective::get(),
            RotationKind::Economic,
            members,
            query_weight,
        )
    }

    fn rotate_building() -> Weight {
        let target = T::CollectiveSize::get();
        let (members, query_weight) = Self::top_subnet_owners(target, T::MinSubnetAge::get());
        Self::apply_rotation(
            T::BuildingCollective::get(),
            RotationKind::Building,
            members,
            query_weight,
        )
    }

    /// Rank coldkeys by total TAO stake (TAO equivalent across all
    /// subnets, including delegated stake). Iterates
    /// `pallet_subtensor::StakingHotkeys` to enumerate coldkeys with
    /// active stake and calls `get_total_stake_for_coldkey` for each.
    /// Returns the top `n` distinct coldkeys, descending by stake.
    ///
    /// Cost is O(coldkeys × hotkeys × subnets); the returned weight
    /// reflects the read-side estimate so the caller (rotation hook)
    /// can bill it to the block.
    pub fn top_validators(n: u32) -> (Vec<T::AccountId>, Weight) {
        let mut weight = Weight::zero();
        let mut entries: Vec<(T::AccountId, subtensor_runtime_common::TaoBalance)> = Vec::new();

        for (coldkey, _) in pallet_subtensor::StakingHotkeys::<T>::iter() {
            // Conservative per-coldkey read estimate — actual cost
            // depends on hotkeys × subnets, which we can't know here
            // without iterating again.
            weight = weight.saturating_add(T::DbWeight::get().reads(8));
            let stake = pallet_subtensor::Pallet::<T>::get_total_stake_for_coldkey(&coldkey);
            entries.push((coldkey, stake));
        }

        entries.sort_by(|a, b| b.1.cmp(&a.1));
        entries.truncate(n as usize);
        let members = entries.into_iter().map(|(c, _)| c).collect::<Vec<_>>();
        (members, weight)
    }

    /// Rank subnet-owner coldkeys by moving-average price, restricted
    /// to subnets registered at least `min_age` blocks ago.
    ///
    /// Multiple subnets owned by the same coldkey are deduplicated to
    /// that coldkey's *highest* moving price — owning more subnets
    /// shouldn't multiply your governance weight beyond a single seat
    /// in the Building collective.
    pub fn top_subnet_owners(
        n: u32,
        min_age: BlockNumberFor<T>,
    ) -> (Vec<T::AccountId>, Weight) {
        let mut weight = Weight::zero();
        let now: u64 = <frame_system::Pallet<T>>::block_number().unique_saturated_into();
        let min_age_u64: u64 = min_age.unique_saturated_into();

        let mut entries: Vec<(T::AccountId, I96F32)> = Vec::new();
        for netuid in pallet_subtensor::Pallet::<T>::get_all_subnet_netuids() {
            // 3 reads: NetworkRegisteredAt + SubnetMovingPrice + SubnetOwner.
            weight = weight.saturating_add(T::DbWeight::get().reads(3));
            let registered_at: u64 = pallet_subtensor::NetworkRegisteredAt::<T>::get(netuid);
            if now.saturating_sub(registered_at) < min_age_u64 {
                continue;
            }
            let price = pallet_subtensor::SubnetMovingPrice::<T>::get(netuid);
            let owner = pallet_subtensor::SubnetOwner::<T>::get(netuid);

            // Dedupe: keep the highest-priced subnet per owner.
            if let Some(existing) = entries.iter_mut().find(|(o, _)| *o == owner) {
                if price > existing.1 {
                    existing.1 = price;
                }
            } else {
                entries.push((owner, price));
            }
        }

        entries.sort_by(|a, b| b.1.cmp(&a.1));
        entries.truncate(n as usize);
        let members = entries.into_iter().map(|(c, _)| c).collect::<Vec<_>>();
        (members, weight)
    }

    /// Push a new membership list into the multi-collective storage for
    /// a ranked collective. We call `reset_members` rather than
    /// reach into storage directly so that:
    /// - duplicate / size validation runs in one place,
    /// - `OnMembersChanged` fires (which the runtime routes to
    ///   signed-voting's `remove_votes_for`, dropping rotated-out
    ///   members' votes from any active poll),
    /// - the standard `MembersReset` event remains the canonical signal.
    ///
    /// `reset_members` enforces `min_members ≤ len ≤ max_members`. The
    /// runtime declares both Economic and Building with `min_members = 0`
    /// (DESIGN.md), so an empty `top_*` result short-circuits cleanly
    /// instead of panicking the rotation hook.
    fn apply_rotation(
        collective_id: <T as pallet_multi_collective::Config>::CollectiveId,
        kind: RotationKind,
        members: Vec<T::AccountId>,
        query_weight: Weight,
    ) -> Weight {
        let len = members.len() as u64;
        let result = pallet_multi_collective::Pallet::<T>::reset_members(
            RawOrigin::Root.into(),
            collective_id,
            members.clone(),
        );

        if let Err(err) = result {
            log::error!(
                target: "runtime::governance-policy",
                "Collective rotation reset_members failed for {:?}: {:?}",
                kind,
                err,
            );
        } else {
            Self::deposit_event(Event::<T>::CollectiveRotated {
                collective_id,
                kind,
                members,
            });
        }

        // 1 read for the existing list + 1 write for the new list +
        // O(len) work in `OnMembersChanged` cleanup. Conservative
        // bound — the actual cleanup cost is voting-pallet-specific.
        query_weight.saturating_add(
            T::DbWeight::get()
                .reads_writes(1, 1)
                .saturating_add(T::DbWeight::get().reads_writes(len, len)),
        )
    }
}
