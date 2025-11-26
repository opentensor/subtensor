"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.extractTime = extractTime;
const MIN_MS = 60 * 1000;
const HR_MS = MIN_MS * 60;
const DAY_MS = HR_MS * 24;
const ZERO = { days: 0, hours: 0, milliseconds: 0, minutes: 0, seconds: 0 };
/** @internal */
function add(a, b) {
    return {
        days: (a.days || 0) + b.days,
        hours: (a.hours || 0) + b.hours,
        milliseconds: (a.milliseconds || 0) + b.milliseconds,
        minutes: (a.minutes || 0) + b.minutes,
        seconds: (a.seconds || 0) + b.seconds
    };
}
/** @internal */
function extractSecs(ms) {
    const s = ms / 1000;
    if (s < 60) {
        const seconds = ~~s;
        return add({ seconds }, extractTime(ms - (seconds * 1000)));
    }
    const m = s / 60;
    if (m < 60) {
        const minutes = ~~m;
        return add({ minutes }, extractTime(ms - (minutes * MIN_MS)));
    }
    const h = m / 60;
    if (h < 24) {
        const hours = ~~h;
        return add({ hours }, extractTime(ms - (hours * HR_MS)));
    }
    const days = ~~(h / 24);
    return add({ days }, extractTime(ms - (days * DAY_MS)));
}
/**
 * @name extractTime
 * @summary Convert a quantity of seconds to Time array representing accumulated {days, minutes, hours, seconds, milliseconds}
 * @example
 * <BR>
 *
 * ```javascript
 * import { extractTime } from '@polkadot/util';
 *
 * const { days, minutes, hours, seconds, milliseconds } = extractTime(6000); // 0, 0, 10, 0, 0
 * ```
 */
function extractTime(milliseconds) {
    return !milliseconds
        ? ZERO
        : milliseconds < 1000
            ? add({ milliseconds }, ZERO)
            : extractSecs(milliseconds);
}
