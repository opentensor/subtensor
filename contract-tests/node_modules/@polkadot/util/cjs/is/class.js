"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isClass = void 0;
const helpers_js_1 = require("./helpers.js");
/**
 * @name isClass
 * Tests if the supplied argument is a Class
 */
exports.isClass = (0, helpers_js_1.isOnFunction)('isPrototypeOf', 'hasOwnProperty');
