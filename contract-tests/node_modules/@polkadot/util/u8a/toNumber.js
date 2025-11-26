/**
 * @name u8aToNumber
 * @summary Creates a number from a Uint8Array object.
 */
export function u8aToNumber(value, { isLe = true, isNegative = false } = {}) {
    // slice + reverse is expensive, however SCALE is LE by default so this is the path
    // we are most interested in (the BE is added for the sake of being comprehensive)
    if (!isLe) {
        value = value.slice().reverse();
    }
    const count = value.length;
    // When the value is a i{8, 16, 24, 32, 40, 40} values and the top-most bit
    // indicates a signed value, we use a two's complement conversion. If one of these
    // flags are not set, we just do a normal unsigned conversion (the same shortcut
    // applies in both the u8aTo{BigInt, Bn} conversions as well)
    if (isNegative && count && (value[count - 1] & 0x80)) {
        switch (count) {
            case 0:
                return 0;
            case 1:
                return (((value[0] ^ 0x0000_00ff) * -1) - 1);
            case 2:
                return ((((value[0] + (value[1] << 8)) ^ 0x0000_ffff) * -1) - 1);
            case 3:
                return ((((value[0] + (value[1] << 8) + (value[2] << 16)) ^ 0x00ff_ffff) * -1) - 1);
            case 4:
                // for the 3rd byte, we don't << 24 - since JS converts all bitwise operators to
                // 32-bit, in the case where the top-most bit is set this yields a negative value
                return ((((value[0] + (value[1] << 8) + (value[2] << 16) + (value[3] * 0x1_00_00_00)) ^ 0xffff_ffff) * -1) - 1);
            case 5:
                return (((((value[0] + (value[1] << 8) + (value[2] << 16) + (value[3] * 0x1_00_00_00)) ^ 0xffff_ffff) + ((value[4] ^ 0xff) * 0x1_00_00_00_00)) * -1) - 1);
            case 6:
                return (((((value[0] + (value[1] << 8) + (value[2] << 16) + (value[3] * 0x1_00_00_00)) ^ 0xffff_ffff) + (((value[4] + (value[5] << 8)) ^ 0x0000_ffff) * 0x1_00_00_00_00)) * -1) - 1);
            default:
                throw new Error('Value more than 48-bits cannot be reliably converted');
        }
    }
    switch (count) {
        case 0:
            return 0;
        case 1:
            return value[0];
        case 2:
            return value[0] + (value[1] << 8);
        case 3:
            return value[0] + (value[1] << 8) + (value[2] << 16);
        case 4:
            // for the 3rd byte, we don't << 24 - since JS converts all bitwise operators to
            // 32-bit, in the case where the top-most bit is set this yields a negative value
            return value[0] + (value[1] << 8) + (value[2] << 16) + (value[3] * 0x1_00_00_00);
        case 5:
            return value[0] + (value[1] << 8) + (value[2] << 16) + ((value[3] + (value[4] << 8)) * 0x1_00_00_00);
        case 6:
            return value[0] + (value[1] << 8) + (value[2] << 16) + ((value[3] + (value[4] << 8) + (value[5] << 16)) * 0x1_00_00_00);
        default:
            throw new Error('Value more than 48-bits cannot be reliably converted');
    }
}
