"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.nToU8a = exports.nToHex = exports.nToBigInt = exports.nSqrt = exports.nMin = exports.nMax = void 0;
const tslib_1 = require("tslib");
/**
 * @summary Utility methods to convert to and from `bigint` objects
 */
var min_js_1 = require("./min.js");
Object.defineProperty(exports, "nMax", { enumerable: true, get: function () { return min_js_1.nMax; } });
Object.defineProperty(exports, "nMin", { enumerable: true, get: function () { return min_js_1.nMin; } });
var sqrt_js_1 = require("./sqrt.js");
Object.defineProperty(exports, "nSqrt", { enumerable: true, get: function () { return sqrt_js_1.nSqrt; } });
var toBigInt_js_1 = require("./toBigInt.js");
Object.defineProperty(exports, "nToBigInt", { enumerable: true, get: function () { return toBigInt_js_1.nToBigInt; } });
var toHex_js_1 = require("./toHex.js");
Object.defineProperty(exports, "nToHex", { enumerable: true, get: function () { return toHex_js_1.nToHex; } });
var toU8a_js_1 = require("./toU8a.js");
Object.defineProperty(exports, "nToU8a", { enumerable: true, get: function () { return toU8a_js_1.nToU8a; } });
tslib_1.__exportStar(require("./consts.js"), exports);
