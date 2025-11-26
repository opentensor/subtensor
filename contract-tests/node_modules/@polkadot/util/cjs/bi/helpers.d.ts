/** @internal */
export declare function createCmp<T>(cmp: (a: T, b: T) => boolean): (...items: T[]) => T;
