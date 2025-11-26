/**
 * @name objectProperty
 * @summary Assign a get property on the input object
 */
export declare function objectProperty<S>(that: object, key: string, getter: (key: string, index: number, self: S) => unknown, getName?: (key: string, index: number) => string, index?: number): void;
/**
 * @name objectProperties
 * @summary Assign get properties on the input object
 */
export declare function objectProperties<S>(that: object, keys: string[], getter: (key: string, index: number, self: S) => unknown, getName?: (key: string, index: number) => string): void;
