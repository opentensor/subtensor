use super::VoteTally;
use frame_support::pallet_prelude::*;
use sp_runtime::Vec;

pub trait SetLike<T> {
    fn contains(&self, item: &T) -> bool;
    fn len(&self) -> u32;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Snapshot the current members. Used by voting pallets that need to freeze the eligible
    /// voter set at poll-start time so mid-poll membership changes can't fabricate votes.
    fn members(&self) -> Vec<T>;
}

pub trait Polls<AccountId> {
    type Index: Parameter + Copy + MaxEncodedLen;
    type VotingScheme: PartialEq;
    type VoterSet: SetLike<AccountId>;

    fn is_ongoing(index: Self::Index) -> bool;
    fn voting_scheme_of(index: Self::Index) -> Option<Self::VotingScheme>;
    fn voter_set_of(index: Self::Index) -> Option<Self::VoterSet>;
    fn on_tally_updated(index: Self::Index, tally: &VoteTally);
}

pub trait PollHooks<PollIndex> {
    fn on_poll_created(poll_index: PollIndex);
    fn on_poll_completed(poll_index: PollIndex);
}

impl<PollIndex> PollHooks<PollIndex> for () {
    fn on_poll_created(_: PollIndex) {}
    fn on_poll_completed(_: PollIndex) {}
}

impl<PollIndex: Clone, A: PollHooks<PollIndex>, B: PollHooks<PollIndex>> PollHooks<PollIndex>
    for (A, B)
{
    fn on_poll_created(poll_index: PollIndex) {
        A::on_poll_created(poll_index.clone());
        B::on_poll_created(poll_index);
    }
    fn on_poll_completed(poll_index: PollIndex) {
        A::on_poll_completed(poll_index.clone());
        B::on_poll_completed(poll_index);
    }
}

impl<PollIndex: Clone, A: PollHooks<PollIndex>, B: PollHooks<PollIndex>, C: PollHooks<PollIndex>>
    PollHooks<PollIndex> for (A, B, C)
{
    fn on_poll_created(poll_index: PollIndex) {
        A::on_poll_created(poll_index.clone());
        B::on_poll_created(poll_index.clone());
        C::on_poll_created(poll_index);
    }
    fn on_poll_completed(poll_index: PollIndex) {
        A::on_poll_completed(poll_index.clone());
        B::on_poll_completed(poll_index.clone());
        C::on_poll_completed(poll_index);
    }
}
