"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.nMin = exports.nMax = void 0;
const helpers_js_1 = require("./helpers.js");
/**
 * @name nMax
 * @summary Finds and returns the highest value in an array of bigint.
 */
exports.nMax = (0, helpers_js_1.createCmp)((a, b) => a > b);
/**
 * @name nMin
 * @summary Finds and returns the lowest value in an array of bigint.
 */
exports.nMin = (0, helpers_js_1.createCmp)((a, b) => a < b);
