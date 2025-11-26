"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.randomAsHex = void 0;
exports.randomAsU8a = randomAsU8a;
const x_randomvalues_1 = require("@polkadot/x-randomvalues");
const helpers_js_1 = require("../helpers.js");
/**
 * @name randomAsU8a
 * @summary Creates a Uint8Array filled with random bytes.
 * @description
 * Returns a `Uint8Array` with the specified (optional) length filled with random bytes.
 * @example
 * <BR>
 *
 * ```javascript
 * import { randomAsU8a } from '@polkadot/util-crypto';
 *
 * randomAsU8a(); // => Uint8Array([...])
 * ```
 */
function randomAsU8a(length = 32) {
    return (0, x_randomvalues_1.getRandomValues)(new Uint8Array(length));
}
/**
 * @name randomAsHex
 * @description Creates a hex string filled with random bytes.
 */
exports.randomAsHex = (0, helpers_js_1.createAsHex)(randomAsU8a);
