"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.Result = void 0;
const Enum_js_1 = require("./Enum.js");
/**
 * @name Result
 * @description
 * A Result maps to the Rust Result type, that can either wrap a success or error value
 */
class Result extends Enum_js_1.Enum {
    constructor(registry, Ok, Err, value) {
        // NOTE This is order-dependent, Ok (with index 0) needs to be first
        // eslint-disable-next-line sort-keys
        super(registry, { Ok, Err }, value);
    }
    static with(Types) {
        return class extends Result {
            constructor(registry, value) {
                super(registry, Types.Ok, Types.Err, value);
            }
        };
    }
    /**
     * @description Returns the wrapper Err value (if isErr)
     */
    get asErr() {
        if (!this.isErr) {
            throw new Error('Cannot extract Err value from Ok result, check isErr first');
        }
        return this.value;
    }
    /**
     * @description Returns the wrapper Ok value (if isOk)
     */
    get asOk() {
        if (!this.isOk) {
            throw new Error('Cannot extract Ok value from Err result, check isOk first');
        }
        return this.value;
    }
    /**
     * @description Checks if the Result has no value
     */
    get isEmpty() {
        return this.isOk && this.value.isEmpty;
    }
    /**
     * @description Checks if the Result wraps an Err value
     */
    get isErr() {
        return !this.isOk;
    }
    /**
     * @description Checks if the Result wraps an Ok value
     */
    get isOk() {
        return this.index === 0;
    }
    /**
     * @description Returns the base runtime type name for this instance
     */
    toRawType() {
        const Types = this._toRawStruct();
        return `Result<${Types.Ok},${Types.Err}>`;
    }
}
exports.Result = Result;
