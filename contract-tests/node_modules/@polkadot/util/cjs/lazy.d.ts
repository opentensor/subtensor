type AnyFn = (...args: unknown[]) => unknown;
/**
 * @name lazyMethod
 * @description
 * Creates a lazy, on-demand getter for the specific value. Upon get the value will be evaluated.
 */
export declare function lazyMethod<T, K, S>(result: Record<string, T> | AnyFn, item: K, creator: (item: K, index: number, self: S) => T, getName?: (item: K, index: number) => string, index?: number): void;
/**
 * @name lazyMethods
 * @description
 * Creates lazy, on-demand getters for the specific values.
 */
export declare function lazyMethods<T, K, S>(result: Record<string, T>, items: readonly K[], creator: (item: K, index: number, self: S) => T, getName?: (item: K, index: number) => string): Record<string, T>;
export {};
