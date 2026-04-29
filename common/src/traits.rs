use super::VoteTally;
use frame_support::pallet_prelude::*;

pub trait SetLike<T> {
    fn contains(&self, item: &T) -> bool;
    fn len(&self) -> u32;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
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

#[impl_trait_for_tuples::impl_for_tuples(10)]
impl<I: Copy> PollHooks<I> for Tuple {
    fn on_poll_created(poll_index: I) {
        for_tuples!( #( Tuple::on_poll_created(poll_index); )* );
    }
    fn on_poll_completed(poll_index: I) {
        for_tuples!( #( Tuple::on_poll_completed(poll_index); )* );
    }
}
