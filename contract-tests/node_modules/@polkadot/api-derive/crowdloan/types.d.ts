import type { Balance } from '@polkadot/types/interfaces';
export interface DeriveContributions {
    blockHash: string;
    contributorsHex: string[];
}
export type DeriveOwnContributions = Record<string, Balance>;
