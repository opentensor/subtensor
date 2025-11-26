import { BN, BN_ONE, BN_TWO, bnToBn, bnToU8a } from '../bn/index.js';
import { u8aConcatStrict } from '../u8a/index.js';
const MAX_U8 = BN_TWO.pow(new BN(8 - 2)).isub(BN_ONE);
const MAX_U16 = BN_TWO.pow(new BN(16 - 2)).isub(BN_ONE);
const MAX_U32 = BN_TWO.pow(new BN(32 - 2)).isub(BN_ONE);
const BL_16 = { bitLength: 16 };
const BL_32 = { bitLength: 32 };
/**
 * @name compactToU8a
 * @description Encodes a number into a compact representation
 * @example
 * <BR>
 *
 * ```javascript
 * import { compactToU8a } from '@polkadot/util';
 *
 * console.log(compactToU8a(511, 32)); // Uint8Array([0b11111101, 0b00000111])
 * ```
 */
export function compactToU8a(value) {
    const bn = bnToBn(value);
    if (bn.lte(MAX_U8)) {
        return new Uint8Array([bn.toNumber() << 2]);
    }
    else if (bn.lte(MAX_U16)) {
        return bnToU8a(bn.shln(2).iadd(BN_ONE), BL_16);
    }
    else if (bn.lte(MAX_U32)) {
        return bnToU8a(bn.shln(2).iadd(BN_TWO), BL_32);
    }
    const u8a = bnToU8a(bn);
    let length = u8a.length;
    // adjust to the minimum number of bytes
    while (u8a[length - 1] === 0) {
        length--;
    }
    if (length < 4) {
        throw new Error('Invalid length, previous checks match anything less than 2^30');
    }
    return u8aConcatStrict([
        // subtract 4 as minimum (also catered for in decoding)
        new Uint8Array([((length - 4) << 2) + 0b11]),
        u8a.subarray(0, length)
    ]);
}
