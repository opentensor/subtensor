"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isBase58 = exports.base58Validate = exports.base58Encode = exports.base58Decode = void 0;
/**
 * @summary Encode and decode base58 values
 */
var bs58_js_1 = require("./bs58.js");
Object.defineProperty(exports, "base58Decode", { enumerable: true, get: function () { return bs58_js_1.base58Decode; } });
Object.defineProperty(exports, "base58Encode", { enumerable: true, get: function () { return bs58_js_1.base58Encode; } });
Object.defineProperty(exports, "base58Validate", { enumerable: true, get: function () { return bs58_js_1.base58Validate; } });
Object.defineProperty(exports, "isBase58", { enumerable: true, get: function () { return bs58_js_1.isBase58; } });
