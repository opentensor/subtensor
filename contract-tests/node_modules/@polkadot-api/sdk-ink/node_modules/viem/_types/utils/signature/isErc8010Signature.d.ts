import type { ErrorType } from '../../errors/utils.js';
import type { Hex } from '../../types/misc.js';
export type IsErc8010SignatureParameters = Hex;
export type IsErc8010SignatureReturnType = boolean;
export type IsErc8010SignatureErrorType = ErrorType;
/** Whether or not the signature is an ERC-8010 formatted signature. */
export declare function isErc8010Signature(signature: IsErc8010SignatureParameters): IsErc8010SignatureReturnType;
//# sourceMappingURL=isErc8010Signature.d.ts.map