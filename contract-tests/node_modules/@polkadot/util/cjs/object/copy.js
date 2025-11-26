"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.objectCopy = objectCopy;
const spread_js_1 = require("./spread.js");
/**
 * @name objectCopy
 * @summary Creates a shallow clone of the input object
 */
function objectCopy(source) {
    return (0, spread_js_1.objectSpread)({}, source);
}
