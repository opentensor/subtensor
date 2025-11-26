/**
 * @name objectSpread
 * @summary Concats all sources into the destination
 * @description Spreads object properties while maintaining object integrity
 */
export declare function objectSpread<T extends object>(dest: object, ...sources: (object | undefined | null)[]): T;
