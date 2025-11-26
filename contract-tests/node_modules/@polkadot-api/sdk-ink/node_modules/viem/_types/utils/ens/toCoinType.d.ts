import { type EnsInvalidChainIdErrorType } from '../../errors/ens.js';
import type { ErrorType } from '../../errors/utils.js';
export type ToCoinTypeError = EnsInvalidChainIdErrorType | ErrorType;
/**
 * @description Converts a chainId to a ENSIP-9 compliant coinType
 *
 * @example
 * toCoinType(10)
 * 2147483658n
 */
export declare function toCoinType(chainId: number): bigint;
//# sourceMappingURL=toCoinType.d.ts.map