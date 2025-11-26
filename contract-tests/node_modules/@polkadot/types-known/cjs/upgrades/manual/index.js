"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.westend = exports.polkadot = exports.kusama = void 0;
var kusama_js_1 = require("./kusama.js");
Object.defineProperty(exports, "kusama", { enumerable: true, get: function () { return kusama_js_1.upgrades; } });
var polkadot_js_1 = require("./polkadot.js");
Object.defineProperty(exports, "polkadot", { enumerable: true, get: function () { return polkadot_js_1.upgrades; } });
var westend_js_1 = require("./westend.js");
Object.defineProperty(exports, "westend", { enumerable: true, get: function () { return westend_js_1.upgrades; } });
