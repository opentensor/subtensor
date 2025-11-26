import type { Observable } from 'rxjs';
import type { AnyNumber } from '@polkadot/types/types';
import type { SignedBlockExtended } from '../type/types.js';
import type { DeriveApi } from '../types.js';
/**
 * @name getBlockByNumber
 * @param {( BN | bigint | Uint8Array | number | string )} blockNumber
 * @description Get a specific block (e.g. rpc.chain.getBlock) and extend it with the author by block number
 * @example
 * ```javascript
 * const { author, block } = await api.derive.chain.getBlockByNumber(123);
 *
 * console.log(`block #${block.header.number} was authored by ${author}`);
 * ```
 */
export declare function getBlockByNumber(instanceId: string, api: DeriveApi): (blockNumber: AnyNumber) => Observable<SignedBlockExtended>;
