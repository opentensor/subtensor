import type { Address } from 'abitype';
import type { ErrorType } from '../../errors/utils.js';
import type { SignedAuthorization } from '../../types/authorization.js';
import type { Hex } from '../../types/misc.js';
import type { OneOf, Prettify } from '../../types/utils.js';
import { type IsErc8010SignatureErrorType } from './isErc8010Signature.js';
export type ParseErc8010SignatureParameters = Hex;
export type ParseErc8010SignatureReturnType = Prettify<OneOf<{
    /** Address of the initializer. */
    address?: Address | undefined;
    /** Authorization signed by the delegatee. */
    authorization: SignedAuthorization;
    /** Data to initialize the delegation. */
    data?: Hex | undefined;
    /** The original signature. */
    signature: Hex;
} | {
    /** The original signature. */
    signature: Hex;
}>>;
export type ParseErc8010SignatureErrorType = IsErc8010SignatureErrorType | ErrorType;
/**
 * @description Parses a hex-formatted ERC-8010 flavoured signature.
 * If the signature is not in ERC-8010 format, then the underlying (original) signature is returned.
 *
 * @param signature ERC-8010 signature in hex format.
 * @returns The parsed ERC-8010 signature.
 */
export declare function parseErc8010Signature(signature: ParseErc8010SignatureParameters): ParseErc8010SignatureReturnType;
//# sourceMappingURL=parseErc8010Signature.d.ts.map