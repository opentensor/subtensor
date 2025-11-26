export const CC_TO_UP = new Array(256);
export const CC_TO_LO = new Array(256);
for (let i = 0, count = CC_TO_UP.length; i < count; i++) {
    CC_TO_LO[i] = String.fromCharCode(i).toLowerCase();
    CC_TO_UP[i] = String.fromCharCode(i).toUpperCase();
}
/** @internal */
function formatAllCaps(w) {
    return w.slice(0, w.length - 1).toLowerCase() + CC_TO_UP[w.charCodeAt(w.length - 1)];
}
/**
 * @internal
 *
 * Inspired by https://stackoverflow.com/a/2970667
 *
 * This is not as optimal as the original SO answer (we split into per-word),
 * however it does pass the tests (which the SO version doesn't) and is still
 * a major improvement over the original camelcase npm package -
 *
 *   camelcase: 20.88 μs/op
 *        this:  1.00 μs/op
 *
 * Caveat of this: only Ascii, but acceptable for the intended usecase
 */
function converter(format) {
    return (value) => {
        const parts = value
            // replace all separators (including consequtive) with spaces
            .replace(/[-_., ]+/g, ' ')
            // we don't want leading or trailing spaces
            .trim()
            // split into words
            .split(' ');
        let result = '';
        for (let i = 0, count = parts.length; i < count; i++) {
            const w = parts[i];
            // apply the formatting
            result += format(/^[\dA-Z]+$/.test(w)
                // all full uppercase + letters are changed to lowercase
                ? w.toLowerCase()
                // all consecutive capitals + letters are changed to lowercase
                // e.g. UUID64 -> uuid64, while preserving splits, eg. NFTOrder -> nftOrder
                : w.replace(/^[\dA-Z]{2,}[^a-z]/, formatAllCaps), i);
        }
        return result;
    };
}
/**
 * @name stringCamelCase
 * @summary Convert a dash/dot/underscore/space separated Ascii string/String to camelCase
 */
export const stringCamelCase = /*#__PURE__*/ converter((w, i) => 
(i ? CC_TO_UP[w.charCodeAt(0)] : CC_TO_LO[w.charCodeAt(0)]) + w.slice(1));
/**
 * @name stringPascalCase
 * @summary Convert a dash/dot/underscore/space separated Ascii string/String to PascalCase
 */
export const stringPascalCase = /*#__PURE__*/ converter((w) => 
CC_TO_UP[w.charCodeAt(0)] + w.slice(1));
