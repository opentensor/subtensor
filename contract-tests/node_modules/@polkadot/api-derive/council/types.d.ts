import type { AccountId, Balance } from '@polkadot/types/interfaces';
export interface DeriveCouncilVote {
    stake: Balance;
    votes: AccountId[];
}
export type DeriveCouncilVotes = [AccountId, DeriveCouncilVote][];
