"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.BigInt = exports.packageInfo = void 0;
const x_global_1 = require("@polkadot/x-global");
var packageInfo_js_1 = require("./packageInfo.js");
Object.defineProperty(exports, "packageInfo", { enumerable: true, get: function () { return packageInfo_js_1.packageInfo; } });
/**
 * @internal
 *
 * There are _still_ some older environments (specifically RN < 0.70), that does
 * not have proper BigInt support - a non-working fallback is provided for those.
 *
 * We detect availability of BigInt upon usage, so this is purely to allow functional
 * compilation & bundling. Since we have operators such as *+-/ top-level, a number-ish
 * result is used here.
 */
function invalidFallback() {
    return Number.NaN;
}
exports.BigInt = (0, x_global_1.extractGlobal)('BigInt', invalidFallback);
