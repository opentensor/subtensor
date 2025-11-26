import type { Address } from 'abitype';
import type { ErrorType } from '../../errors/utils.js';
import type { SignedAuthorization } from '../../types/authorization.js';
import type { ByteArray, Hex } from '../../types/misc.js';
type To = 'bytes' | 'hex';
export type SerializeErc8010SignatureParameters<to extends To = 'hex'> = {
    /** Address of the initializer. */
    address?: Address | undefined;
    /** Authorization signed by the delegatee. */
    authorization: SignedAuthorization;
    /** Data to initialize the delegation. */
    data?: Hex | undefined;
    /** The original signature. */
    signature: Hex;
    to?: to | To | undefined;
};
export type SerializeErc8010SignatureReturnType<to extends To = 'hex'> = (to extends 'hex' ? Hex : never) | (to extends 'bytes' ? ByteArray : never);
export type SerializeErc8010SignatureErrorType = ErrorType;
/**
 * @description Serializes a ERC-8010 flavoured signature into hex format.
 *
 * @param signature ERC-8010 signature in object format.
 * @returns ERC-8010 signature in hex format.
 */
export declare function serializeErc8010Signature<to extends To = 'hex'>(parameters: SerializeErc8010SignatureParameters<to>): SerializeErc8010SignatureReturnType<to>;
export {};
//# sourceMappingURL=serializeErc8010Signature.d.ts.map