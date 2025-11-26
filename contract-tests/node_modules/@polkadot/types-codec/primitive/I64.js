import { Int } from '../base/Int.js';
/**
 * @name i64
 * @description
 * A 64-bit signed integer
 */
export class i64 extends Int.with(64) {
    // NOTE without this, we cannot properly determine extensions
    __IntType = 'i64';
}
