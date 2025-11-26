"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.formatElapsed = formatElapsed;
const toBn_js_1 = require("../bn/toBn.js");
/** @internal */
function formatValue(elapsed) {
    if (elapsed < 15) {
        return `${elapsed.toFixed(1)}s`;
    }
    else if (elapsed < 60) {
        return `${elapsed | 0}s`;
    }
    else if (elapsed < 3600) {
        return `${elapsed / 60 | 0}m`;
    }
    return `${elapsed / 3600 | 0}h`;
}
/**
 * @name formatElapsed
 * @description Formats an elapsed value into s, m, h or day segments
 */
function formatElapsed(now, value) {
    const tsNow = now?.getTime() || 0;
    const tsValue = value instanceof Date
        ? value.getTime()
        : (0, toBn_js_1.bnToBn)(value).toNumber();
    return (tsNow && tsValue)
        ? formatValue(Math.max(Math.abs(tsNow - tsValue), 0) / 1000)
        : '0.0s';
}
