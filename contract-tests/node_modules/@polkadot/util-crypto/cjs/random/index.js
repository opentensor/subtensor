"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.randomAsU8a = exports.randomAsHex = exports.randomAsNumber = void 0;
/**
 * @summary Returns a sequence of secure random bytes in a variety of formats
 */
var asNumber_js_1 = require("./asNumber.js");
Object.defineProperty(exports, "randomAsNumber", { enumerable: true, get: function () { return asNumber_js_1.randomAsNumber; } });
var asU8a_js_1 = require("./asU8a.js");
Object.defineProperty(exports, "randomAsHex", { enumerable: true, get: function () { return asU8a_js_1.randomAsHex; } });
Object.defineProperty(exports, "randomAsU8a", { enumerable: true, get: function () { return asU8a_js_1.randomAsU8a; } });
