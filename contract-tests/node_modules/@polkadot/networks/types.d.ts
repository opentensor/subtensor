import type { RegistryEntry } from '@substrate/ss58-registry';
import type { HexString } from '@polkadot/util/types';
export type Icon = 'beachball' | 'empty' | 'jdenticon' | 'polkadot' | 'substrate';
export type KnownIcon = Record<string, Icon>;
export type KnownLedger = Record<string, number>;
export type KnownGenesis = Record<string, HexString[]>;
export type KnownSubstrate = RegistryEntry;
export type KnownTestnet = Record<string, true>;
export interface SubstrateNetwork extends KnownSubstrate {
    /** The genesisHash for the chain */
    genesisHash: HexString[];
    /** Does the chain has support for Ledger devices */
    hasLedgerSupport: boolean;
    /** The IdentityIcon to use for the chain */
    icon: Icon;
    /** Flag set when we don't include this chain */
    isIgnored: boolean;
    /** Flag to indicate a testnet */
    isTestnet: boolean;
    /** The Ledger-specific/required slip44 for the chain */
    slip44?: number | null;
}
export interface Network extends SubstrateNetwork {
    /** The network assigned to this chain */
    network: string;
}
export interface Ss58Registry {
    registry: KnownSubstrate[];
    specification: string;
    schema: Record<keyof KnownSubstrate, string>;
}
