"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isCodec = isCodec;
const helpers_js_1 = require("./helpers.js");
const checkCodec = /*#__PURE__*/ (0, helpers_js_1.isOnObject)('toHex', 'toHuman', 'toU8a');
const checkRegistry = /*#__PURE__*/ (0, helpers_js_1.isOnObject)('get');
function isCodec(value) {
    return checkCodec(value) && checkRegistry(value.registry);
}
