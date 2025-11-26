import { isClass } from './class.js';
/**
 * @name isChildClass
 * @summary Tests if the child extends the parent Class
 * @description
 * Checks to see if the child Class extends the parent Class
 * @example
 * <BR>
 *
 * ```javascript
 * import { isChildClass } from '@polkadot/util';
 *
 * console.log('isChildClass', isChildClass(BN, BN); // => true
 * console.log('isChildClass', isChildClass(BN, Uint8Array); // => false
 * ```
 */
export function isChildClass(Parent, Child) {
    // https://stackoverflow.com/questions/30993434/check-if-a-constructor-inherits-another-in-es6/30993664
    return isClass(Child) && isClass(Parent)
        // eslint-disable-next-line no-prototype-builtins
        ? Parent === Child || Parent.isPrototypeOf(Child)
        : false;
}
