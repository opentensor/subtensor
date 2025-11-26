"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getSeparator = getSeparator;
/**
 * Get the decimal and thousand separator of a locale
 * @param locale
 * @returns {decimal: string, thousand: string}
 */
function getSeparator(locale) {
    return {
        decimal: (0.1).toLocaleString(locale, { useGrouping: false }).charAt(1),
        thousand: (1000).toLocaleString(locale, { useGrouping: true }).replace(/\d/g, '').charAt(0)
    };
}
