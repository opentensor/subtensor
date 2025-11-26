import * as Bytes from '../Bytes.js';
import type * as Errors from '../Errors.js';
import * as Hex from '../Hex.js';
/** @internal */
export declare const integerToAlphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
/** @internal */
export declare const alphabetToInteger: Readonly<Record<string, bigint>>;
/** @internal */
export declare function from(value: Hex.Hex | Bytes.Bytes): string;
/** @internal */
export declare namespace from {
    type ErrorType = Errors.GlobalErrorType;
}
/** @internal */
//# sourceMappingURL=base58.d.ts.map