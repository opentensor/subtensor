"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.formatNumber = formatNumber;
const toBn_js_1 = require("../bn/toBn.js");
const formatDecimal_js_1 = require("./formatDecimal.js");
const getSeparator_js_1 = require("./getSeparator.js");
/**
 * @name formatNumber
 * @description Formats a number into string format with thousand separators
 */
function formatNumber(value, { locale = 'en' } = {}) {
    const { thousand } = (0, getSeparator_js_1.getSeparator)(locale);
    return (0, formatDecimal_js_1.formatDecimal)((0, toBn_js_1.bnToBn)(value).toString(), thousand);
}
