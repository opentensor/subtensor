"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.WebSocket = exports.packageInfo = void 0;
const tslib_1 = require("tslib");
const ws_1 = tslib_1.__importDefault(require("ws"));
const x_global_1 = require("@polkadot/x-global");
var packageInfo_js_1 = require("./packageInfo.js");
Object.defineProperty(exports, "packageInfo", { enumerable: true, get: function () { return packageInfo_js_1.packageInfo; } });
exports.WebSocket = (0, x_global_1.extractGlobal)('WebSocket', ws_1.default);
