"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const tslib_1 = require("tslib");
require("./packageDetect.js");
const bundle_js_1 = require("./bundle.js");
tslib_1.__exportStar(require("./bundle.js"), exports);
exports.default = bundle_js_1.Keyring;
