"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isBuffer = isBuffer;
const x_global_1 = require("@polkadot/x-global");
const has_js_1 = require("../has.js");
const function_js_1 = require("./function.js");
/**
 * @name isBuffer
 * @summary Tests for a `Buffer` object instance.
 * @description
 * Checks to see if the input object is an instance of `Buffer`.
 * @example
 * <BR>
 *
 * ```javascript
 * import { isBuffer } from '@polkadot/util';
 *
 * console.log('isBuffer', isBuffer(Buffer.from([]))); // => true
 * ```
 */
function isBuffer(value) {
    // we do check a function first, since it is slightly faster than isBuffer itself
    return has_js_1.hasBuffer && !!value && (0, function_js_1.isFunction)(value.readDoubleLE) && x_global_1.xglobal.Buffer.isBuffer(value);
}
