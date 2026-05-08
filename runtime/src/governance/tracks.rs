//! Static list of referenda tracks. Track 0 is the triumvirate
//! approval track; track 1 is the collective oversight (Review) track.

use pallet_referenda::{
    ApprovalAction, DecisionStrategy, MAX_TRACK_NAME_LEN, Track as RefTrack,
    TrackInfo as RefTrackInfo, TracksInfo as RefTracksInfo,
};
use sp_runtime::Perbill;
use subtensor_runtime_common::pad_name;

use crate::{
    AccountId, BlockNumber, GovernanceCollectiveId, GovernanceCollectiveInitialDelay,
    GovernanceMemberSet, GovernanceTriumvirateDecisionPeriod, GovernanceVotingScheme, RuntimeCall,
};

pub struct SubtensorTracks;

impl RefTracksInfo<[u8; MAX_TRACK_NAME_LEN], AccountId, RuntimeCall, BlockNumber>
    for SubtensorTracks
{
    type Id = u8;
    type ProposerSet = GovernanceMemberSet;
    type VotingScheme = GovernanceVotingScheme;
    type VoterSet = GovernanceMemberSet;

    fn tracks() -> impl Iterator<
        Item = RefTrack<
            Self::Id,
            [u8; MAX_TRACK_NAME_LEN],
            BlockNumber,
            Self::ProposerSet,
            Self::VoterSet,
            Self::VotingScheme,
        >,
    > {
        [
            RefTrack {
                id: 0u8,
                info: RefTrackInfo {
                    name: pad_name(b"triumvirate"),
                    proposer_set: Some(GovernanceMemberSet::Single(
                        GovernanceCollectiveId::Proposers,
                    )),
                    voter_set: GovernanceMemberSet::Single(GovernanceCollectiveId::Triumvirate),
                    voting_scheme: GovernanceVotingScheme::Signed,
                    decision_strategy: DecisionStrategy::PassOrFail {
                        decision_period: GovernanceTriumvirateDecisionPeriod::get(),
                        // 2/3 approval
                        approve_threshold: Perbill::from_rational(2u32, 3u32),
                        reject_threshold: Perbill::from_rational(2u32, 3u32),
                        // Approved triumvirate decisions hand off to the
                        // collective review track (track 1) so the wider
                        // body can fast-track or cancel before enactment.
                        on_approval: ApprovalAction::Review { track: 1 },
                    },
                },
            },
            // `proposer_set: None` is load-bearing: it makes track 1
            // reachable only via Track 0's `ApprovalAction::Review` handoff.
            // Setting it to `Some(_)` would let a proposer schedule a root
            // call for auto-dispatch at `now + initial_delay`, bypassing
            // Triumvirate approval.
            RefTrack {
                id: 1u8,
                info: RefTrackInfo {
                    name: pad_name(b"review"),
                    proposer_set: None,
                    voter_set: GovernanceMemberSet::Union(alloc::vec![
                        GovernanceCollectiveId::Economic,
                        GovernanceCollectiveId::Building,
                    ]),
                    voting_scheme: GovernanceVotingScheme::Signed,
                    decision_strategy: DecisionStrategy::Adjustable {
                        initial_delay: GovernanceCollectiveInitialDelay::get(),
                        fast_track_threshold: Perbill::from_percent(75),
                        cancel_threshold: Perbill::from_percent(51),
                    },
                },
            },
        ]
        .into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pallet_referenda::TracksInfo;

    #[test]
    fn track_0_triumvirate_is_directly_submittable() {
        let track_0 = SubtensorTracks::tracks()
            .find(|t| t.id == 0u8)
            .expect("track 0 (triumvirate) must exist");

        assert!(
            track_0.info.proposer_set.is_some(),
            "track 0 must have a proposer_set; without it there is no \
             on-chain entry point into governance."
        );
    }

    #[test]
    fn track_1_review_is_not_directly_submittable() {
        let track_1 = SubtensorTracks::tracks()
            .find(|t| t.id == 1u8)
            .expect("track 1 (review) must exist");

        assert!(
            track_1.info.proposer_set.is_none(),
            "track 1 must have proposer_set: None; Some(_) would let a \
             proposer schedule a root call without Triumvirate approval."
        );
    }
}
