use frame_support::pallet_prelude::*;

/// Handler for when the members of a collective have changed.
pub trait OnMembersChanged<CollectiveId, AccountId> {
    /// A collective's members have changed, `incoming` members have joined and
    /// `outgoing` members have left.
    fn on_members_changed(
        collective_id: CollectiveId,
        incoming: &[AccountId],
        outgoing: &[AccountId],
    );
    /// Worst-case upper bound on `on_members_changed`'s weight. The
    /// implementation is responsible for bounding its own iteration over
    /// `incoming`/`outgoing` against the relevant `MaxMembers` constant.
    fn weight() -> Weight;
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

    fn weight() -> Weight {
        #[allow(clippy::let_and_return)]
        let mut weight = Weight::zero();
        for_tuples!( #( weight.saturating_accrue(Tuple::weight()); )* );
        weight
    }
}
