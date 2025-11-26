import { EnsInvalidChainIdError, } from '../../errors/ens.js';
const SLIP44_MSB = 0x80000000;
/**
 * @description Converts a chainId to a ENSIP-9 compliant coinType
 *
 * @example
 * toCoinType(10)
 * 2147483658n
 */
export function toCoinType(chainId) {
    if (chainId === 1)
        return 60n;
    if (chainId >= SLIP44_MSB || chainId < 0)
        throw new EnsInvalidChainIdError({ chainId });
    return BigInt((0x80000000 | chainId) >>> 0);
}
//# sourceMappingURL=toCoinType.js.map