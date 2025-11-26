import { UInt } from '../base/UInt.js';
/**
 * @name u64
 * @description
 * A 64-bit unsigned integer
 */
export class u64 extends UInt.with(64) {
    // NOTE without this, we cannot properly determine extensions
    __UIntType = 'u64';
}
