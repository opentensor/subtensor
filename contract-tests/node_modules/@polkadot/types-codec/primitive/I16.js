import { Int } from '../base/Int.js';
/**
 * @name i16
 * @description
 * A 16-bit signed integer
 */
export class i16 extends Int.with(16) {
    // NOTE without this, we cannot properly determine extensions
    __IntType = 'i16';
}
