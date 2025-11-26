import { bnToBn } from '../bn/toBn.js';
import { formatDecimal } from './formatDecimal.js';
import { getSeparator } from './getSeparator.js';
/**
 * @name formatNumber
 * @description Formats a number into string format with thousand separators
 */
export function formatNumber(value, { locale = 'en' } = {}) {
    const { thousand } = getSeparator(locale);
    return formatDecimal(bnToBn(value).toString(), thousand);
}
