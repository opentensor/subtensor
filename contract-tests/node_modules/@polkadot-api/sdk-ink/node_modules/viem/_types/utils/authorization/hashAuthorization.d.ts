import type { ErrorType } from '../../errors/utils.js';
import type { AuthorizationRequest } from '../../types/authorization.js';
import type { ByteArray, Hex } from '../../types/misc.js';
import { type ConcatHexErrorType } from '../data/concat.js';
import { type HexToBytesErrorType } from '../encoding/toBytes.js';
import { type NumberToHexErrorType } from '../encoding/toHex.js';
import { type ToRlpErrorType } from '../encoding/toRlp.js';
import { type Keccak256ErrorType } from '../hash/keccak256.js';
type To = 'hex' | 'bytes';
export type HashAuthorizationParameters<to extends To> = AuthorizationRequest & {
    /** Output format. @default "hex" */
    to?: to | To | undefined;
};
export type HashAuthorizationReturnType<to extends To> = (to extends 'bytes' ? ByteArray : never) | (to extends 'hex' ? Hex : never);
export type HashAuthorizationErrorType = Keccak256ErrorType | ConcatHexErrorType | ToRlpErrorType | NumberToHexErrorType | HexToBytesErrorType | ErrorType;
/**
 * Computes an Authorization hash in [EIP-7702 format](https://eips.ethereum.org/EIPS/eip-7702): `keccak256('0x05' || rlp([chain_id, address, nonce]))`.
 */
export declare function hashAuthorization<to extends To = 'hex'>(parameters: HashAuthorizationParameters<to>): HashAuthorizationReturnType<to>;
export {};
//# sourceMappingURL=hashAuthorization.d.ts.map