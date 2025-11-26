"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.encodeToCurve = exports.hashToCurve = exports.secp384r1 = exports.p384 = void 0;
const nist_ts_1 = require("./nist.js");
exports.p384 = nist_ts_1.p384;
exports.secp384r1 = nist_ts_1.p384;
exports.hashToCurve = (() => nist_ts_1.p384_hasher.hashToCurve)();
exports.encodeToCurve = (() => nist_ts_1.p384_hasher.encodeToCurve)();
/** @deprecated Use `import { p384_hasher } from "@noble/curves/nist"` module. */
//# sourceMappingURL=p384.js.map