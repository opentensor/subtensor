"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.encodeToCurve = exports.hashToCurve = exports.secp256r1 = exports.p256 = void 0;
const nist_ts_1 = require("./nist.js");
exports.p256 = nist_ts_1.p256;
exports.secp256r1 = nist_ts_1.p256;
exports.hashToCurve = (() => nist_ts_1.p256_hasher.hashToCurve)();
exports.encodeToCurve = (() => nist_ts_1.p256_hasher.encodeToCurve)();
//# sourceMappingURL=p256.js.map