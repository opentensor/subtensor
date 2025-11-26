"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.stringToU8a = exports.stringToHex = exports.stringShorten = exports.stringUpperFirst = exports.stringLowerFirst = exports.stringPascalCase = exports.stringCamelCase = void 0;
/**
 * @summary Utility methods to convert to work with `string` values
 */
var camelCase_js_1 = require("./camelCase.js");
Object.defineProperty(exports, "stringCamelCase", { enumerable: true, get: function () { return camelCase_js_1.stringCamelCase; } });
Object.defineProperty(exports, "stringPascalCase", { enumerable: true, get: function () { return camelCase_js_1.stringPascalCase; } });
var lowerFirst_js_1 = require("./lowerFirst.js");
Object.defineProperty(exports, "stringLowerFirst", { enumerable: true, get: function () { return lowerFirst_js_1.stringLowerFirst; } });
Object.defineProperty(exports, "stringUpperFirst", { enumerable: true, get: function () { return lowerFirst_js_1.stringUpperFirst; } });
var shorten_js_1 = require("./shorten.js");
Object.defineProperty(exports, "stringShorten", { enumerable: true, get: function () { return shorten_js_1.stringShorten; } });
var toHex_js_1 = require("./toHex.js");
Object.defineProperty(exports, "stringToHex", { enumerable: true, get: function () { return toHex_js_1.stringToHex; } });
var toU8a_js_1 = require("./toU8a.js");
Object.defineProperty(exports, "stringToU8a", { enumerable: true, get: function () { return toU8a_js_1.stringToU8a; } });
