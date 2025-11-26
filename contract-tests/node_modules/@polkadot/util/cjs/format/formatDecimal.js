"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.formatDecimal = formatDecimal;
const NUMBER_REGEX = new RegExp('(\\d+?)(?=(\\d{3})+(?!\\d)|$)', 'g');
/**
 * @name formatDecimal
 * @description Formats a number into string format with thousand separators
 */
function formatDecimal(value, separator = ',') {
    // We can do this by adjusting the regx, however for the sake of clarity
    // we rather strip and re-add the negative sign in the output
    const isNegative = value[0].startsWith('-');
    const matched = isNegative
        ? value.substring(1).match(NUMBER_REGEX)
        : value.match(NUMBER_REGEX);
    return matched
        ? `${isNegative ? '-' : ''}${matched.join(separator)}`
        : value;
}
