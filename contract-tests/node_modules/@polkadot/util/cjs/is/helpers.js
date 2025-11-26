"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isOn = isOn;
exports.isOnFunction = isOnFunction;
exports.isOnObject = isOnObject;
const function_js_1 = require("./function.js");
const object_js_1 = require("./object.js");
function isOn(...fns) {
    return (value) => ((0, object_js_1.isObject)(value) || (0, function_js_1.isFunction)(value)) &&
        fns.every((f) => (0, function_js_1.isFunction)(value[f]));
}
function isOnFunction(...fns) {
    return (value) => (0, function_js_1.isFunction)(value) &&
        fns.every((f) => (0, function_js_1.isFunction)(value[f]));
}
function isOnObject(...fns) {
    return (value) => (0, object_js_1.isObject)(value) &&
        fns.every((f) => (0, function_js_1.isFunction)(value[f]));
}
