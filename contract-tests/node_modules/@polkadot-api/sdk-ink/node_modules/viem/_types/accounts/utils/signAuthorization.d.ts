import type { ErrorType } from '../../errors/utils.js';
import type { AuthorizationRequest, SignedAuthorization } from '../../types/authorization.js';
import type { Hex } from '../../types/misc.js';
import type { Prettify } from '../../types/utils.js';
import { type HashAuthorizationErrorType } from '../../utils/authorization/hashAuthorization.js';
import { type SignErrorType, type SignParameters, type SignReturnType } from './sign.js';
type To = 'object' | 'bytes' | 'hex';
export type SignAuthorizationParameters<to extends To = 'object'> = AuthorizationRequest & {
    /** The private key to sign with. */
    privateKey: Hex;
    to?: SignParameters<to>['to'] | undefined;
};
export type SignAuthorizationReturnType<to extends To = 'object'> = Prettify<to extends 'object' ? SignedAuthorization : SignReturnType<to>>;
export type SignAuthorizationErrorType = SignErrorType | HashAuthorizationErrorType | ErrorType;
/**
 * Signs an Authorization hash in [EIP-7702 format](https://eips.ethereum.org/EIPS/eip-7702): `keccak256('0x05' || rlp([chain_id, address, nonce]))`.
 */
export declare function signAuthorization<to extends To = 'object'>(parameters: SignAuthorizationParameters<to>): Promise<SignAuthorizationReturnType<to>>;
export {};
//# sourceMappingURL=signAuthorization.d.ts.map