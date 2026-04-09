use super::VoteTally;
use frame_support::pallet_prelude::*;

pub trait SetLike<T> {
    fn contains(&self, item: &T) -> bool;
    fn len(&self) -> u32;
}

pub trait Polls<AccountId> {
    type Index: Parameter + Copy;
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
