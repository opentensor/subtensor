import { Float } from '../native/Float.js';
/**
 * @name f64
 * @description
 * A 64-bit float
 */
export class f64 extends Float.with(64) {
    // NOTE without this, we cannot properly determine extensions
    __FloatType = 'f64';
}
