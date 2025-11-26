"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isObservable = void 0;
const helpers_js_1 = require("./helpers.js");
/**
 * @name isBObservable
 * @summary Tests for a `Observable` object instance.
 * @description
 * Checks to see if the input object is an instance of `BN` (bn.js).
 * @example
 * <BR>
 *
 * ```javascript
 * import { isObservable } from '@polkadot/util';
 *
 * console.log('isObservable', isObservable(...));
 * ```
 */
exports.isObservable = (0, helpers_js_1.isOn)('next');
