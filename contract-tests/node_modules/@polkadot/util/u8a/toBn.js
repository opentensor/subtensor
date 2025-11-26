import { BN } from '../bn/bn.js';
/**
 * @name u8aToBn
 * @summary Creates a BN from a Uint8Array object.
 * @description
 * `UInt8Array` input values return the actual BN. `null` or `undefined` values returns an `0x0` value.
 * @param value The value to convert
 * @param options Options to pass while converting
 * @param options.isLe Convert using Little Endian (default)
 * @param options.isNegative Convert using two's complement
 * @example
 * <BR>
 *
 * ```javascript
 * import { u8aToBn } from '@polkadot/util';
 *
 * u8aToHex(new Uint8Array([0x68, 0x65, 0x6c, 0x6c, 0xf])); // 0x68656c0f
 * ```
 */
export function u8aToBn(value, { isLe = true, isNegative = false } = {}) {
    // slice + reverse is expensive, however SCALE is LE by default so this is the path
    // we are most interested in (the BE is added for the sake of being comprehensive)
    if (!isLe) {
        value = value.slice().reverse();
    }
    const count = value.length;
    // shortcut for <= u48 values - in this case the manual conversion
    // here seems to be more efficient than passing the full array
    if (isNegative && count && (value[count - 1] & 0x80)) {
        // Most common case i{8, 16, 32} default LE SCALE-encoded
        // For <= 32, we also optimize the xor to a single op
        switch (count) {
            case 0:
                return new BN(0);
            case 1:
                return new BN(((value[0] ^ 0x0000_00ff) * -1) - 1);
            case 2:
                return new BN((((value[0] + (value[1] << 8)) ^ 0x0000_ffff) * -1) - 1);
            case 3:
                return new BN((((value[0] + (value[1] << 8) + (value[2] << 16)) ^ 0x00ff_ffff) * -1) - 1);
            case 4:
                // for the 3rd byte, we don't << 24 - since JS converts all bitwise operators to
                // 32-bit, in the case where the top-most bit is set this yields a negative value
                return new BN((((value[0] + (value[1] << 8) + (value[2] << 16) + (value[3] * 0x1_00_00_00)) ^ 0xffff_ffff) * -1) - 1);
            case 5:
                return new BN(((((value[0] + (value[1] << 8) + (value[2] << 16) + (value[3] * 0x1_00_00_00)) ^ 0xffff_ffff) + ((value[4] ^ 0xff) * 0x1_00_00_00_00)) * -1) - 1);
            case 6:
                return new BN(((((value[0] + (value[1] << 8) + (value[2] << 16) + (value[3] * 0x1_00_00_00)) ^ 0xffff_ffff) + (((value[4] + (value[5] << 8)) ^ 0x0000_ffff) * 0x1_00_00_00_00)) * -1) - 1);
            default:
                return new BN(value, 'le').fromTwos(count * 8);
        }
    }
    // Most common case - u{8, 16, 32} default LE SCALE-encoded
    //
    // There are some slight benefits in unrolling this specific loop,
    // however it comes with diminishing returns since here the actual
    // `new BN` does seem to take up the bulk of the time
    switch (count) {
        case 0:
            return new BN(0);
        case 1:
            return new BN(value[0]);
        case 2:
            return new BN(value[0] + (value[1] << 8));
        case 3:
            return new BN(value[0] + (value[1] << 8) + (value[2] << 16));
        case 4:
            // for the 3rd byte, we don't << 24 - since JS converts all bitwise operators to
            // 32-bit, in the case where the top-most bit is set this yields a negative value
            return new BN(value[0] + (value[1] << 8) + (value[2] << 16) + (value[3] * 0x1_00_00_00));
        case 5:
            return new BN(value[0] + (value[1] << 8) + (value[2] << 16) + ((value[3] + (value[4] << 8)) * 0x1_00_00_00));
        case 6:
            return new BN(value[0] + (value[1] << 8) + (value[2] << 16) + ((value[3] + (value[4] << 8) + (value[5] << 16)) * 0x1_00_00_00));
        default:
            return new BN(value, 'le');
    }
}
