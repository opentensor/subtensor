/**
 * @name arrayUnzip
 * @description Splits a single [K, V][] into [K[], V[]]
 */
export declare function arrayUnzip<K, V>(entries: readonly [K, V][]): [K[], V[]];
