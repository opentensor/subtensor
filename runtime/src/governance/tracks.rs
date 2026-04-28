//! Static list of referenda tracks. Track 0 is the triumvirate
//! approval track; track 1 is the collective oversight (Review) track.

use pallet_referenda::{
    DecisionStrategy, MAX_TRACK_NAME_LEN, Track as RefTrack, TrackInfo as RefTrackInfo,
    TracksInfo as RefTracksInfo,
};
use sp_runtime::Perbill;

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
        fn name(s: &[u8]) -> [u8; MAX_TRACK_NAME_LEN] {
            let mut out = [0u8; MAX_TRACK_NAME_LEN];
            out.iter_mut()
                .zip(s.iter())
                .for_each(|(dst, src)| *dst = *src);
            out
        }

        [
            RefTrack {
                id: 0u8,
                info: RefTrackInfo {
                    name: name(b"triumvirate"),
                    proposer_set: GovernanceMemberSet::Single(GovernanceCollectiveId::Proposers),
                    voter_set: GovernanceMemberSet::Single(GovernanceCollectiveId::Triumvirate),
                    voting_scheme: GovernanceVotingScheme::Signed,
                    decision_strategy: DecisionStrategy::PassOrFail {
                        decision_period: GovernanceTriumvirateDecisionPeriod::get(),
                        // 2/3 approval
                        approve_threshold: Perbill::from_rational(2u32, 3u32),
                        reject_threshold: Perbill::from_rational(2u32, 3u32),
                    },
                },
            },
            RefTrack {
                id: 1u8,
                info: RefTrackInfo {
                    name: name(b"review"),
                    proposer_set: GovernanceMemberSet::Single(GovernanceCollectiveId::Proposers),
                    voter_set: GovernanceMemberSet::Union(alloc::vec![
                        GovernanceCollectiveId::Economic,
                        GovernanceCollectiveId::Building,
                    ]),
                    voting_scheme: GovernanceVotingScheme::Signed,
                    decision_strategy: DecisionStrategy::Adjustable {
                        initial_delay: GovernanceCollectiveInitialDelay::get(),
                        fast_track_threshold: Perbill::from_percent(67),
                        reject_threshold: Perbill::from_percent(51),
                    },
                },
            },
        ]
        .into_iter()
    }
}
