import type { Observable } from 'rxjs';
import type { GenericExtrinsic } from '@polkadot/types';
import type { EventRecord, Hash } from '@polkadot/types/interfaces';
import type { DeriveApi } from '../types.js';
interface ExtrinsicInfo {
    blockHash: Hash | string;
    blockNumber: number;
    extrinsic: GenericExtrinsic;
    events: EventRecord[];
    success: boolean;
}
interface ExtrinsicsInfo {
    blockHash: Hash | string;
    blockNumber: number;
    extrinsics: {
        events: EventRecord[];
        extrinsic: GenericExtrinsic;
        success: boolean;
    }[];
}
/**
 * @name extrinsicInfo
 * @param { Hash } at The block hash to query at.
 * @param { Uint8Array | string } transactionHash A transaction hash as U8 array or string.
 * @description Retrieves the extrinsic information and its events.
 * @example
 * ```javascript
 * const blockHash = api.registry.createType(
 *   'Hash',
 *   '0xb772e4949d2f3eb5ba356aa43f885cc4f9097ee9812c5436543f3846a0491729'
 * );
 * const extrinsicInfo = await api.derive.tx.extrinsicInfo(
 *   blockHash,
 *   '0xcd96520b05e0c4648ea365f3f063f27c5cdd8be10d41a1c44566428c91f37dcb'
 * );
 *
 * console.log(extrinsicInfo.extrinsic.toHuman());
 * ```
 */
export declare function extrinsicInfo(instanceId: string, api: DeriveApi): (at: Hash, transactionHash: Uint8Array | string) => Observable<ExtrinsicInfo | null>;
/**
 * @name accountExtrinsics
 * @description Retrieves information about every extrinsic submitted by an account at a given block.
 * @param { Hash } at The block hash to query at.
 * @param { Uint8Array | strings } accountId The account identifier to query.
 * @example
 * ```javascript
   const blockHash = api.registry.createType(
    'Hash',
    '0xb772e4949d2f3eb5ba356aa43f885cc4f9097ee9812c5436543f3846a0491729'
  );
  const extrinsicsInfo = await api.derive.tx.accountExtrinsics(
    blockHash,
    '0x21895DdfD4640b4e0aDCa2865b907f2CE6e6B777'
  );

  console.log(extrinsicsInfo.extrinsics[0]).extrinsic.toHuman();
 * ```
 */
export declare function accountExtrinsics(instanceId: string, api: DeriveApi): (at: Hash, accountId: Uint8Array | string) => Observable<ExtrinsicsInfo>;
export {};
