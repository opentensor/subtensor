"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.encodeToCurve = exports.hashToCurve = exports.secp521r1 = exports.p521 = void 0;
const nist_ts_1 = require("./nist.js");
exports.p521 = nist_ts_1.p521;
exports.secp521r1 = nist_ts_1.p521;
exports.hashToCurve = (() => nist_ts_1.p521_hasher.hashToCurve)();
exports.encodeToCurve = (() => nist_ts_1.p521_hasher.encodeToCurve)();
//# sourceMappingURL=p521.js.map