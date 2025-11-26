import { isOnFunction } from './helpers.js';
/**
 * @name isClass
 * Tests if the supplied argument is a Class
 */
export const isClass = /*#__PURE__*/ isOnFunction('isPrototypeOf', 'hasOwnProperty');
