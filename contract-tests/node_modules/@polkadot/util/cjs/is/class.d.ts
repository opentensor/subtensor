import type { Class } from '../types.js';
/**
 * @name isClass
 * Tests if the supplied argument is a Class
 */
export declare const isClass: <T extends Class>(value?: unknown) => value is T;
