import { BaseError } from './base.js';
export class BundleFailedError extends BaseError {
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
//# sourceMappingURL=calls.js.map