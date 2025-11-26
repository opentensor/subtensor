import type { OneOf, Pretty } from './types.js';
type BaseErrorArgs = Pretty<{
    docsPath?: string | undefined;
    metaMessages?: string[] | undefined;
} & OneOf<{
    details?: string | undefined;
} | {
    cause?: BaseError | Error;
}>>;
export declare class BaseError extends Error {
    details: string;
    docsPath?: string | undefined;
    metaMessages?: string[] | undefined;
    shortMessage: string;
    name: string;
    constructor(shortMessage: string, args?: BaseErrorArgs);
}
export {};
//# sourceMappingURL=errors.d.ts.map