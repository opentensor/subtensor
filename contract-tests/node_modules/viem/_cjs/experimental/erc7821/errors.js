"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.FunctionSelectorNotRecognizedError = exports.ExecuteUnsupportedError = void 0;
const base_js_1 = require("../../errors/base.js");
class ExecuteUnsupportedError extends base_js_1.BaseError {
    constructor() {
        super('ERC-7821 execution is not supported.', {
            name: 'ExecuteUnsupportedError',
        });
    }
}
exports.ExecuteUnsupportedError = ExecuteUnsupportedError;
class FunctionSelectorNotRecognizedError extends base_js_1.BaseError {
    constructor() {
        super('Function is not recognized.', {
            metaMessages: [
                'This could be due to any of the following:',
                '  - The contract does not have the function,',
                '  - The address is not a contract.',
            ],
            name: 'FunctionSelectorNotRecognizedError',
        });
    }
}
exports.FunctionSelectorNotRecognizedError = FunctionSelectorNotRecognizedError;
//# sourceMappingURL=errors.js.map