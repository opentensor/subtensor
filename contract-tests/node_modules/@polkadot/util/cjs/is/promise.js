"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isPromise = void 0;
const helpers_js_1 = require("./helpers.js");
exports.isPromise = (0, helpers_js_1.isOnObject)('catch', 'then');
