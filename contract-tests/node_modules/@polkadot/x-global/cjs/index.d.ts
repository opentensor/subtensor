export { packageInfo } from './packageInfo.js';
type GlobalThis = typeof globalThis & {
    process?: {
        env?: Record<string, string>;
    };
    [key: string]: unknown;
};
type GlobalNames = keyof typeof globalThis;
type GlobalType<N extends GlobalNames> = typeof globalThis[N];
/**
 * A cross-environment implementation for globalThis
 */
export declare const xglobal: GlobalThis;
/**
 * Extracts a known global from the environment, applying a fallback if not found
 */
export declare function extractGlobal<N extends GlobalNames, T extends GlobalType<N>>(name: N, fallback: unknown): T;
/**
 * Expose a value as a known global, if not already defined
 */
export declare function exposeGlobal<N extends GlobalNames, T extends GlobalType<N>>(name: N, fallback: unknown): void;
