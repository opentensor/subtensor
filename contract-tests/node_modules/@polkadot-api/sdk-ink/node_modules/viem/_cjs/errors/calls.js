"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.BundleFailedError = void 0;
const base_js_1 = require("./base.js");
class BundleFailedError extends base_js_1.BaseError {
    constructor(result) {
        super(`Call bundle failed with status: ${result.statusCode}`, {
            name: 'BundleFailedError',
        });
        Object.defineProperty(this, "result", {
            enumerable: true,
            configurable: true,
            writable: true,
            value: void 0
        });
        this.result = result;
    }
}
exports.BundleFailedError = BundleFailedError;
//# sourceMappingURL=calls.js.map