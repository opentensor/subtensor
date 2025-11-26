import * as Errors from '../Errors.js';
/** @internal */
export declare function withTimeout<data>(fn: withTimeout.Fn<data>, options: withTimeout.Options): Promise<data>;
/** @internal */
export declare namespace withTimeout {
    type Fn<data> = ({ signal, }: {
        signal: AbortController['signal'] | null;
    }) => Promise<data>;
    type Options = {
        errorInstance?: Error | undefined;
        timeout: number;
        signal?: boolean | undefined;
    };
    type ErrorType = TimeoutError | Errors.GlobalErrorType;
}
/** @internal */
/**
 * Thrown when an operation times out.
 * @internal
 */
export declare class TimeoutError extends Errors.BaseError {
    readonly name = "Promise.TimeoutError";
    constructor();
}
//# sourceMappingURL=promise.d.ts.map