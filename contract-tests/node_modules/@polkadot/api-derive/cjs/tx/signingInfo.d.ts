import type { Observable } from 'rxjs';
import type { Header, Index } from '@polkadot/types/interfaces';
import type { AnyNumber, Codec, IExtrinsicEra } from '@polkadot/types/types';
import type { DeriveApi } from '../types.js';
interface Result {
    header: Header | null;
    mortalLength: number;
    nonce: Index;
}
/**
 * @name signingInfo
 * @description Retrieves signing-related information for an account, including the nonce, block header, and mortal length.
 * @param {string} address The account address for which signing information is needed.
 * @param { BN | bigint | Uint8Array | number | string } nonce? (Optional) The nonce to use. If `undefined`, the latest nonce is retrieved.
 * @param { IExtrinsicEra | number } era? (Optional) The transaction era.
 * @example
 * ```javascript
 * const info = await api.derive.tx.signingInfo(
 *   "14mM9FRDDtwSYicjNxSvMfQkap8o4m9zHq7hNW4JpbSL4PPU"
 * );
 * console.log(info);
 * ```
 */
export declare function signingInfo(_instanceId: string, api: DeriveApi): (address: string, nonce?: AnyNumber | Codec, era?: IExtrinsicEra | number) => Observable<Result>;
export {};
