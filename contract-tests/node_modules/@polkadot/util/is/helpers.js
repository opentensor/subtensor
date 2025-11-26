import { isFunction } from './function.js';
import { isObject } from './object.js';
export function isOn(...fns) {
    return (value) => (isObject(value) || isFunction(value)) &&
        fns.every((f) => isFunction(value[f]));
}
export function isOnFunction(...fns) {
    return (value) => isFunction(value) &&
        fns.every((f) => isFunction(value[f]));
}
export function isOnObject(...fns) {
    return (value) => isObject(value) &&
        fns.every((f) => isFunction(value[f]));
}
