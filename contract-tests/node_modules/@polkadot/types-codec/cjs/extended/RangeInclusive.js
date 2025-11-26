"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.RangeInclusive = void 0;
const Range_js_1 = require("./Range.js");
class RangeInclusive extends Range_js_1.Range {
    constructor(registry, Type, value) {
        super(registry, Type, value, { rangeName: 'RangeInclusive' });
    }
    static with(Type) {
        return class extends RangeInclusive {
            constructor(registry, value) {
                super(registry, Type, value);
            }
        };
    }
}
exports.RangeInclusive = RangeInclusive;
