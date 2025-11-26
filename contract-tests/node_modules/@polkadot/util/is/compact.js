import { isOnObject } from './helpers.js';
/**
 * @name isCompact
 * @summary Tests for SCALE-Compact-like object instance.
 */
export const isCompact = /*#__PURE__*/ isOnObject('toBigInt', 'toBn', 'toNumber', 'unwrap');
