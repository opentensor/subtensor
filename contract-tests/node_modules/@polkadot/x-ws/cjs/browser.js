"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.WebSocket = exports.packageInfo = void 0;
const x_global_1 = require("@polkadot/x-global");
var packageInfo_js_1 = require("./packageInfo.js");
Object.defineProperty(exports, "packageInfo", { enumerable: true, get: function () { return packageInfo_js_1.packageInfo; } });
exports.WebSocket = x_global_1.xglobal.WebSocket;
