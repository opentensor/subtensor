import type { HexString } from '@polkadot/util/types';
export type ChainUpgradesRaw = [blockNumber: number, specVersion: number][];
export type ChainUpgradesExpanded = [blockNumber: number, specVersion: number, runtimeApis: [apiHash: HexString, apiVersion: number][]][];
