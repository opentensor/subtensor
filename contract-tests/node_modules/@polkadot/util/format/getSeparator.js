/**
 * Get the decimal and thousand separator of a locale
 * @param locale
 * @returns {decimal: string, thousand: string}
 */
export function getSeparator(locale) {
    return {
        decimal: (0.1).toLocaleString(locale, { useGrouping: false }).charAt(1),
        thousand: (1000).toLocaleString(locale, { useGrouping: true }).replace(/\d/g, '').charAt(0)
    };
}
