import { BN } from '../bn/index.js';
import { u8aToBn, u8aToU8a } from '../u8a/index.js';
/**
 * @name compactFromU8a
 * @description Retrives the offset and encoded length from a compact-prefixed value
 * @example
 * <BR>
 *
 * ```javascript
 * import { compactFromU8a } from '@polkadot/util';
 *
 * const [offset, length] = compactFromU8a(new Uint8Array([254, 255, 3, 0]));
 *
 * console.log('value offset=', offset, 'length=', length); // 4, 0xffff
 * ```
 */
export function compactFromU8a(input) {
    const u8a = u8aToU8a(input);
    // The u8a is manually converted here for 1, 2 & 4 lengths, it is 2x faster
    // than doing an additional call to u8aToBn (as with variable length)
    switch (u8a[0] & 0b11) {
        case 0b00:
            return [1, new BN(u8a[0] >>> 2)];
        case 0b01:
            return [2, new BN((u8a[0] + (u8a[1] << 8)) >>> 2)];
        case 0b10:
            // for the 3rd byte, we don't << 24 - since JS converts all bitwise operators to
            // 32-bit, in the case where the top-most bit is set this yields a negative value
            return [4, new BN((u8a[0] + (u8a[1] << 8) + (u8a[2] << 16) + (u8a[3] * 0x1_00_00_00)) >>> 2)];
        // 0b11
        default: {
            // add 5 to shifted (4 for base length, 1 for this byte)
            const offset = (u8a[0] >>> 2) + 5;
            // we unroll the loop
            switch (offset) {
                // there still could be 4 bytes data, similar to 0b10 above (with offsets)
                case 5:
                    // for the 3rd byte, we don't << 24 - since JS converts all bitwise operators to
                    // 32-bit, in the case where the top-most bit is set this yields a negative value
                    return [5, new BN(u8a[1] + (u8a[2] << 8) + (u8a[3] << 16) + (u8a[4] * 0x1_00_00_00))];
                case 6:
                    return [6, new BN(u8a[1] + (u8a[2] << 8) + (u8a[3] << 16) + ((u8a[4] + (u8a[5] << 8)) * 0x1_00_00_00))];
                // 6 bytes data is the maximum, 48 bits (56 would overflow)
                case 7:
                    return [7, new BN(u8a[1] + (u8a[2] << 8) + (u8a[3] << 16) + ((u8a[4] + (u8a[5] << 8) + (u8a[6] << 16)) * 0x1_00_00_00))];
                // for anything else, use the non-unrolled version
                default:
                    return [offset, u8aToBn(u8a.subarray(1, offset))];
            }
        }
    }
}
/**
 * @name compactFromU8aLim
 * @description A limited version of [[compactFromU8a]], accepting only Uint8Array inputs for values <= 48 bits
 */
export function compactFromU8aLim(u8a) {
    // The u8a is manually converted here for 1, 2 & 4 lengths, it is 2x faster
    // than doing an additional call to u8aToBn (as with variable length)
    switch (u8a[0] & 0b11) {
        case 0b00:
            return [1, u8a[0] >>> 2];
        case 0b01:
            return [2, (u8a[0] + (u8a[1] << 8)) >>> 2];
        case 0b10:
            // for the 3rd byte, we don't << 24 - since JS converts all bitwise operators to
            // 32-bit, in the case where the top-most bit is set this yields a negative value
            return [4, (u8a[0] + (u8a[1] << 8) + (u8a[2] << 16) + (u8a[3] * 0x1_00_00_00)) >>> 2];
        // 0b11
        default: {
            // add 5 to shifted (4 for base length, 1 for this byte)
            // we unroll the loop
            switch ((u8a[0] >>> 2) + 5) {
                // there still could be 4 bytes data, similar to 0b10 above (with offsets)
                case 5:
                    return [5, u8a[1] + (u8a[2] << 8) + (u8a[3] << 16) + (u8a[4] * 0x1_00_00_00)];
                case 6:
                    return [6, u8a[1] + (u8a[2] << 8) + (u8a[3] << 16) + ((u8a[4] + (u8a[5] << 8)) * 0x1_00_00_00)];
                // 6 bytes data is the maximum, 48 bits (56 would overflow)
                case 7:
                    return [7, u8a[1] + (u8a[2] << 8) + (u8a[3] << 16) + ((u8a[4] + (u8a[5] << 8) + (u8a[6] << 16)) * 0x1_00_00_00)];
                // for anything else, we are above the actual MAX_SAFE_INTEGER - bail out
                default:
                    throw new Error('Compact input is > Number.MAX_SAFE_INTEGER');
            }
        }
    }
}
