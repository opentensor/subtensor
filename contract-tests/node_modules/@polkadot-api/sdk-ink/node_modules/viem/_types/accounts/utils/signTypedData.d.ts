import type { TypedData } from 'abitype';
import type { ErrorType } from '../../errors/utils.js';
import type { Hex } from '../../types/misc.js';
import type { TypedDataDefinition } from '../../types/typedData.js';
import { type HashTypedDataErrorType } from '../../utils/signature/hashTypedData.js';
import { type SignErrorType } from './sign.js';
export type SignTypedDataParameters<typedData extends TypedData | Record<string, unknown> = TypedData, primaryType extends keyof typedData | 'EIP712Domain' = keyof typedData> = TypedDataDefinition<typedData, primaryType> & {
    /** The private key to sign with. */
    privateKey: Hex;
};
export type SignTypedDataReturnType = Hex;
export type SignTypedDataErrorType = HashTypedDataErrorType | SignErrorType | ErrorType;
/**
 * @description Signs typed data and calculates an Ethereum-specific signature in [https://eips.ethereum.org/EIPS/eip-712](https://eips.ethereum.org/EIPS/eip-712):
 * `sign(keccak256("\x19\x01" ‖ domainSeparator ‖ hashStruct(message)))`.
 *
 * @returns The signature.
 */
export declare function signTypedData<const typedData extends TypedData | Record<string, unknown>, primaryType extends keyof typedData | 'EIP712Domain'>(parameters: SignTypedDataParameters<typedData, primaryType>): Promise<SignTypedDataReturnType>;
//# sourceMappingURL=signTypedData.d.ts.map