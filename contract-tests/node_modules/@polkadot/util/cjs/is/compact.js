"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isCompact = void 0;
const helpers_js_1 = require("./helpers.js");
/**
 * @name isCompact
 * @summary Tests for SCALE-Compact-like object instance.
 */
exports.isCompact = (0, helpers_js_1.isOnObject)('toBigInt', 'toBn', 'toNumber', 'unwrap');
