"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.prime = prime;
const rxjs_1 = require("rxjs");
const util_1 = require("@polkadot/util");
const helpers_js_1 = require("./helpers.js");
function prime(section) {
    return (0, helpers_js_1.withSection)(section, (query) => () => (0, util_1.isFunction)(query?.prime)
        ? query.prime().pipe((0, rxjs_1.map)((o) => o.unwrapOr(null)))
        : (0, rxjs_1.of)(null));
}
