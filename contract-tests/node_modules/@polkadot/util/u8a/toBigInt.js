import { BigInt } from '@polkadot/x-bigint';
import { _1n } from '../bi/consts.js';
const U8_MAX = BigInt(256);
const U16_MAX = BigInt(256 * 256);
const U64_MAX = BigInt('0x10000000000000000');
/**
 * @name u8aToBigInt
 * @summary Creates a BigInt from a Uint8Array object.
 */
export function u8aToBigInt(value, { isLe = true, isNegative = false } = {}) {
    // slice + reverse is expensive, however SCALE is LE by default so this is the path
    // we are most interested in (the BE is added for the sake of being comprehensive)
    if (!isLe) {
        value = value.slice().reverse();
    }
    const count = value.length;
    if (isNegative && count && (value[count - 1] & 0x80)) {
        switch (count) {
            case 0:
                return BigInt(0);
            case 1:
                return BigInt(((value[0] ^ 0x0000_00ff) * -1) - 1);
            case 2:
                return BigInt((((value[0] + (value[1] << 8)) ^ 0x0000_ffff) * -1) - 1);
            case 4:
                return BigInt((((value[0] + (value[1] << 8) + (value[2] << 16) + (value[3] * 0x1_00_00_00)) ^ 0xffff_ffff) * -1) - 1);
        }
        const dvI = new DataView(value.buffer, value.byteOffset);
        if (count === 8) {
            return dvI.getBigInt64(0, true);
        }
        let result = BigInt(0);
        const mod = count % 2;
        for (let i = count - 2; i >= mod; i -= 2) {
            result = (result * U16_MAX) + BigInt(dvI.getUint16(i, true) ^ 0xffff);
        }
        if (mod) {
            result = (result * U8_MAX) + BigInt(value[0] ^ 0xff);
        }
        return (result * -_1n) - _1n;
    }
    switch (count) {
        case 0:
            return BigInt(0);
        case 1:
            return BigInt(value[0]);
        case 2:
            return BigInt(value[0] + (value[1] << 8));
        case 4:
            return BigInt(value[0] + (value[1] << 8) + (value[2] << 16) + (value[3] * 0x1_00_00_00));
    }
    const dvI = new DataView(value.buffer, value.byteOffset);
    switch (count) {
        case 8:
            return dvI.getBigUint64(0, true);
        case 16:
            return (dvI.getBigUint64(8, true) * U64_MAX) + dvI.getBigUint64(0, true);
        default: {
            let result = BigInt(0);
            const mod = count % 2;
            for (let i = count - 2; i >= mod; i -= 2) {
                result = (result * U16_MAX) + BigInt(dvI.getUint16(i, true));
            }
            if (mod) {
                result = (result * U8_MAX) + BigInt(value[0]);
            }
            return result;
        }
    }
}
