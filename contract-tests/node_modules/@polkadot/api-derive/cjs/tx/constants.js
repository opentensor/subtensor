"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.MORTAL_PERIOD = exports.MAX_FINALITY_LAG = exports.FALLBACK_PERIOD = exports.FALLBACK_MAX_HASH_COUNT = void 0;
const util_1 = require("@polkadot/util");
exports.FALLBACK_MAX_HASH_COUNT = 250;
exports.FALLBACK_PERIOD = new util_1.BN(6 * 1000);
exports.MAX_FINALITY_LAG = new util_1.BN(5);
exports.MORTAL_PERIOD = new util_1.BN(5 * 60 * 1000);
