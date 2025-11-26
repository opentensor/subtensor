"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.base64Trim = exports.base64Pad = exports.isBase64 = exports.base64Validate = exports.base64Encode = exports.base64Decode = void 0;
/**
 * @summary Encode and decode base64 values
 */
var bs64_js_1 = require("./bs64.js");
Object.defineProperty(exports, "base64Decode", { enumerable: true, get: function () { return bs64_js_1.base64Decode; } });
Object.defineProperty(exports, "base64Encode", { enumerable: true, get: function () { return bs64_js_1.base64Encode; } });
Object.defineProperty(exports, "base64Validate", { enumerable: true, get: function () { return bs64_js_1.base64Validate; } });
Object.defineProperty(exports, "isBase64", { enumerable: true, get: function () { return bs64_js_1.isBase64; } });
var pad_js_1 = require("./pad.js");
Object.defineProperty(exports, "base64Pad", { enumerable: true, get: function () { return pad_js_1.base64Pad; } });
var trim_js_1 = require("./trim.js");
Object.defineProperty(exports, "base64Trim", { enumerable: true, get: function () { return trim_js_1.base64Trim; } });
