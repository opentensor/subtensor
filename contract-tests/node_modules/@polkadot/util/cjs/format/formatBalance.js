"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.formatBalance = void 0;
const toBn_js_1 = require("../bn/toBn.js");
const boolean_js_1 = require("../is/boolean.js");
const formatDecimal_js_1 = require("./formatDecimal.js");
const getSeparator_js_1 = require("./getSeparator.js");
const si_js_1 = require("./si.js");
const DEFAULT_DECIMALS = 0;
const DEFAULT_UNIT = si_js_1.SI[si_js_1.SI_MID].text;
let defaultDecimals = DEFAULT_DECIMALS;
let defaultUnit = DEFAULT_UNIT;
function _formatBalance(input, { decimals = defaultDecimals, forceUnit, locale = 'en', withAll = false, withSi = true, withSiFull = false, withUnit = true, withZero = true } = {}) {
    // we only work with string inputs here - convert anything
    // into the string-only value
    let text = (0, toBn_js_1.bnToBn)(input).toString();
    if (text.length === 0 || text === '0') {
        return '0';
    }
    // strip the negative sign so we can work with clean groupings, re-add this in the
    // end when we return the result (from here on we work with positive numbers)
    let sign = '';
    if (text[0].startsWith('-')) {
        sign = '-';
        text = text.substring(1);
    }
    // We start at midpoint (8) minus 1 - this means that values display as
    // 123.4567 instead of 0.1234 k (so we always have the most relevant).
    const si = (0, si_js_1.calcSi)(text, decimals, forceUnit);
    const mid = text.length - (decimals + si.power);
    const pre = mid <= 0 ? '0' : text.substring(0, mid);
    // get the post from the midpoint onward and then first add max decimals
    // before trimming to the correct (calculated) amount of decimals again
    let post = text
        .padStart(mid < 0 ? decimals : 1, '0')
        .substring(mid < 0 ? 0 : mid)
        .padEnd(withAll ? Math.max(decimals, 4) : 4, '0')
        .substring(0, withAll ? Math.max(4, decimals + si.power) : 4);
    // remove all trailing 0's (if required via flag)
    if (!withZero) {
        let end = post.length - 1;
        // This looks inefficient, however it is better to do the checks and
        // only make one final slice than it is to do it in multiples
        do {
            if (post[end] === '0') {
                end--;
            }
        } while (post[end] === '0');
        post = post.substring(0, end + 1);
    }
    // the display unit
    const unit = (0, boolean_js_1.isBoolean)(withUnit)
        ? si_js_1.SI[si_js_1.SI_MID].text
        : withUnit;
    // format the units for display based on the flags
    const units = withSi || withSiFull
        ? si.value === '-'
            ? withUnit
                ? ` ${unit}`
                : ''
            : ` ${withSiFull ? `${si.text}${withUnit ? ' ' : ''}` : si.value}${withUnit ? unit : ''}`
        : '';
    const { decimal, thousand } = (0, getSeparator_js_1.getSeparator)(locale);
    return `${sign}${(0, formatDecimal_js_1.formatDecimal)(pre, thousand)}${post && `${decimal}${post}`}${units}`;
}
exports.formatBalance = _formatBalance;
exports.formatBalance.calcSi = (text, decimals = defaultDecimals) => (0, si_js_1.calcSi)(text, decimals);
exports.formatBalance.findSi = si_js_1.findSi;
exports.formatBalance.getDefaults = () => {
    return {
        decimals: defaultDecimals,
        unit: defaultUnit
    };
};
exports.formatBalance.getOptions = (decimals = defaultDecimals) => {
    return si_js_1.SI.filter(({ power }) => power < 0
        ? (decimals + power) >= 0
        : true);
};
exports.formatBalance.setDefaults = ({ decimals, unit }) => {
    defaultDecimals = (Array.isArray(decimals)
        ? decimals[0]
        : decimals) ?? defaultDecimals;
    defaultUnit = (Array.isArray(unit)
        ? unit[0]
        : unit) ?? defaultUnit;
    si_js_1.SI[si_js_1.SI_MID].text = defaultUnit;
};
