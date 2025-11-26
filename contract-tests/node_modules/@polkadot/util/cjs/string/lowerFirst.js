"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.stringUpperFirst = exports.stringLowerFirst = void 0;
const camelCase_js_1 = require("./camelCase.js");
/** @internal */
function converter(map) {
    return (value) => value
        ? map[value.charCodeAt(0)] + value.slice(1)
        : '';
}
/**
 * @name stringLowerFirst
 * @summary Lowercase the first letter of a string
 * @description
 * Lowercase the first letter of a string
 * @example
 * <BR>
 *
 * ```javascript
 * import { stringLowerFirst } from '@polkadot/util';
 *
 * stringLowerFirst('ABC'); // => 'aBC'
 * ```
 */
exports.stringLowerFirst = converter(camelCase_js_1.CC_TO_LO);
/**
 * @name stringUpperFirst
 * @summary Uppercase the first letter of a string
 * @description
 * Lowercase the first letter of a string
 * @example
 * <BR>
 *
 * ```javascript
 * import { stringUpperFirst } from '@polkadot/util';
 *
 * stringUpperFirst('abc'); // => 'Abc'
 * ```
 */
exports.stringUpperFirst = converter(camelCase_js_1.CC_TO_UP);
