import type { Observable } from 'rxjs';
import type { Compact, Vec } from '@polkadot/types';
import type { AccountId, BlockNumber, Header } from '@polkadot/types/interfaces';
import type { DeriveApi } from '../types.js';
export type BlockNumberDerive = (instanceId: string, api: DeriveApi) => () => Observable<BlockNumber>;
export declare function createBlockNumberDerive<T extends {
    number: Compact<BlockNumber> | BlockNumber;
}>(fn: (api: DeriveApi) => Observable<T>): BlockNumberDerive;
export declare function getAuthorDetails(api: DeriveApi, header: Header, blockHash?: Uint8Array | string): Observable<[Header, Vec<AccountId> | null, AccountId | null]>;
