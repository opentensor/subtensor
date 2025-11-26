"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.loggerFormat = loggerFormat;
exports.logger = logger;
const x_global_1 = require("@polkadot/x-global");
const formatDate_js_1 = require("./format/formatDate.js");
const bn_js_1 = require("./is/bn.js");
const buffer_js_1 = require("./is/buffer.js");
const function_js_1 = require("./is/function.js");
const object_js_1 = require("./is/object.js");
const u8a_js_1 = require("./is/u8a.js");
const toHex_js_1 = require("./u8a/toHex.js");
const toU8a_js_1 = require("./u8a/toU8a.js");
const noop_js_1 = require("./noop.js");
const logTo = {
    debug: 'log',
    error: 'error',
    log: 'log',
    warn: 'warn'
};
function formatOther(value) {
    if (value && (0, object_js_1.isObject)(value) && value.constructor === Object) {
        const result = {};
        for (const [k, v] of Object.entries(value)) {
            result[k] = loggerFormat(v);
        }
        return result;
    }
    return value;
}
function loggerFormat(value) {
    if (Array.isArray(value)) {
        return value.map(loggerFormat);
    }
    else if ((0, bn_js_1.isBn)(value)) {
        return value.toString();
    }
    else if ((0, u8a_js_1.isU8a)(value) || (0, buffer_js_1.isBuffer)(value)) {
        return (0, toHex_js_1.u8aToHex)((0, toU8a_js_1.u8aToU8a)(value));
    }
    return formatOther(value);
}
function formatWithLength(maxLength) {
    return (v) => {
        if (maxLength <= 0) {
            return v;
        }
        const r = `${v}`;
        return r.length < maxLength
            ? v
            : `${r.substring(0, maxLength)} ...`;
    };
}
function apply(log, type, values, maxSize = -1) {
    if (values.length === 1 && (0, function_js_1.isFunction)(values[0])) {
        const fnResult = values[0]();
        return apply(log, type, Array.isArray(fnResult) ? fnResult : [fnResult], maxSize);
    }
    console[logTo[log]]((0, formatDate_js_1.formatDate)(new Date()), type, ...values
        .map(loggerFormat)
        .map(formatWithLength(maxSize)));
}
function isDebugOn(e, type) {
    return !!e && (e === '*' ||
        type === e ||
        (e.endsWith('*') &&
            type.startsWith(e.slice(0, -1))));
}
function isDebugOff(e, type) {
    return !!e && (e.startsWith('-') &&
        (type === e.slice(1) ||
            (e.endsWith('*') &&
                type.startsWith(e.slice(1, -1)))));
}
function getDebugFlag(env, type) {
    let flag = false;
    for (const e of env) {
        if (isDebugOn(e, type)) {
            flag = true;
        }
        else if (isDebugOff(e, type)) {
            flag = false;
        }
    }
    return flag;
}
function parseEnv(type) {
    const maxSize = parseInt(x_global_1.xglobal.process?.env?.['DEBUG_MAX'] || '-1', 10);
    return [
        getDebugFlag((x_global_1.xglobal.process?.env?.['DEBUG'] || '').toLowerCase().split(','), type),
        isNaN(maxSize)
            ? -1
            : maxSize
    ];
}
/**
 * @name Logger
 * @summary Creates a consistent log interface for messages
 * @description
 * Returns a `Logger` that has `.log`, `.error`, `.warn` and `.debug` (controlled with environment `DEBUG=typeA,typeB`) methods. Logging is done with a consistent prefix (type of logger, date) followed by the actual message using the underlying console.
 * @example
 * <BR>
 *
 * ```javascript
 * import { logger } from '@polkadot/util';
 *
 * const l = logger('test');
 * ```
 */
function logger(origin) {
    const type = `${origin.toUpperCase()}:`.padStart(16);
    const [isDebug, maxSize] = parseEnv(origin.toLowerCase());
    return {
        debug: isDebug
            ? (...values) => apply('debug', type, values, maxSize)
            : noop_js_1.noop,
        error: (...values) => apply('error', type, values),
        log: (...values) => apply('log', type, values),
        noop: noop_js_1.noop,
        warn: (...values) => apply('warn', type, values)
    };
}
