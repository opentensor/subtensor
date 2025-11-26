/**
 * Generic EthereumJS error class with metadata attached
 *
 * Kudos to https://github.com/ChainSafe/lodestar monorepo
 * for the inspiration :-)
 * See: https://github.com/ChainSafe/lodestar/blob/unstable/packages/utils/src/errors.ts
 */
export type EthereumJSErrorMetaData = Record<string, string | number | null>;
export type EthereumJSErrorObject = {
    message: string;
    stack: string;
    className: string;
    type: EthereumJSErrorMetaData;
};
export declare const DEFAULT_ERROR_CODE = "ETHEREUMJS_DEFAULT_ERROR_CODE";
/**
 * Generic EthereumJS error with attached metadata
 */
export declare class EthereumJSError<T extends {
    code: string;
}> extends Error {
    type: T;
    constructor(type: T, message?: string, stack?: string);
    getMetadata(): EthereumJSErrorMetaData;
    /**
     * Get the metadata and the stacktrace for the error.
     */
    toObject(): EthereumJSErrorObject;
}
/**
 * @deprecated Use `EthereumJSError` with a set error code instead
 * @param message Optional error message
 * @param stack Optional stack trace
 * @returns
 */
export declare function EthereumJSErrorWithoutCode(message?: string, stack?: string): EthereumJSError<{
    code: string;
}>;
//# sourceMappingURL=errors.d.ts.map