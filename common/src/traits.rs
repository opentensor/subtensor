use super::VoteTally;
use frame_support::pallet_prelude::*;

pub trait SetLike<T> {
    fn contains(&self, item: &T) -> bool;
    fn len(&self) -> u32;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Poll provider seen from the voting pallet's side. Carries the
/// read-only queries plus the tally-update notification fired when a
/// vote moves the tally.
pub trait Polls<AccountId> {
    type Index: Parameter + Copy;
    type VotingScheme: PartialEq;
    type VoterSet: SetLike<AccountId>;

    fn is_ongoing(index: Self::Index) -> bool;
    fn voting_scheme_of(index: Self::Index) -> Option<Self::VotingScheme>;
    fn voter_set_of(index: Self::Index) -> Option<Self::VoterSet>;

    fn on_tally_updated(index: Self::Index, tally: &VoteTally);
    /// Worst-case upper bound on `on_tally_updated`'s weight.
    fn on_tally_updated_weight() -> Weight;
}

/// Notification fired when a poll is created.
pub trait OnPollCreated<PollIndex> {
    fn on_poll_created(poll_index: PollIndex);
    /// Returns the worst-case upper bound on `on_poll_created`'s weight.
    fn weight() -> Weight;
}

/// Notification fired when a poll reaches a terminal status.
pub trait OnPollCompleted<PollIndex> {
    fn on_poll_completed(poll_index: PollIndex);
    /// Returns the worst-case upper bound on `on_poll_completed`'s weight.
    fn weight() -> Weight;
}

#[impl_trait_for_tuples::impl_for_tuples(10)]
impl<I: Copy> OnPollCreated<I> for Tuple {
    fn on_poll_created(poll_index: I) {
        for_tuples!( #( Tuple::on_poll_created(poll_index); )* );
    }

    fn weight() -> Weight {
        #[allow(clippy::let_and_return)]
        let mut weight = Weight::zero();
        for_tuples!( #( weight.saturating_accrue(Tuple::weight()); )* );
        weight
    }
}

#[impl_trait_for_tuples::impl_for_tuples(10)]
impl<I: Copy> OnPollCompleted<I> for Tuple {
    fn on_poll_completed(poll_index: I) {
        for_tuples!( #( Tuple::on_poll_completed(poll_index); )* );
    }

    fn weight() -> Weight {
        #[allow(clippy::let_and_return)]
        let mut weight = Weight::zero();
        for_tuples!( #( weight.saturating_accrue(Tuple::weight()); )* );
        weight
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
