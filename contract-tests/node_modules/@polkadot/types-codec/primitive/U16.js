import { UInt } from '../base/UInt.js';
/**
 * @name u16
 * @description
 * A 16-bit unsigned integer
 */
export class u16 extends UInt.with(16) {
    // NOTE without this, we cannot properly determine extensions
    __UIntType = 'u16';
}
