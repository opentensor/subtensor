import type { Hash, ReferendumInfoTo239 } from '@polkadot/types/interfaces';
import type { FrameSupportPreimagesBounded, PalletDemocracyReferendumInfo, PalletDemocracyReferendumStatus, PalletDemocracyVoteThreshold } from '@polkadot/types/lookup';
import type { Option } from '@polkadot/types-codec';
import type { HexString } from '@polkadot/util/types';
import type { DeriveReferendum, DeriveReferendumVote, DeriveReferendumVotes } from '../types.js';
import { BN } from '@polkadot/util';
interface ApproxState {
    votedAye: BN;
    votedNay: BN;
    votedTotal: BN;
}
export declare function compareRationals(n1: BN, d1: BN, n2: BN, d2: BN): boolean;
export declare function calcPassing(threshold: PalletDemocracyVoteThreshold, sqrtElectorate: BN, state: ApproxState): boolean;
export declare function calcVotes(sqrtElectorate: BN, referendum: DeriveReferendum, votes: DeriveReferendumVote[]): DeriveReferendumVotes;
export declare function getStatus(info: Option<PalletDemocracyReferendumInfo | ReferendumInfoTo239>): PalletDemocracyReferendumStatus | ReferendumInfoTo239 | null;
export declare function getImageHashBounded(hash: Uint8Array | string | Hash | FrameSupportPreimagesBounded): HexString;
export declare function getImageHash(status: PalletDemocracyReferendumStatus | ReferendumInfoTo239): HexString;
export {};
