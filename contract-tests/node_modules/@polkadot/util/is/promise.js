import { isOnObject } from './helpers.js';
export const isPromise = /*#__PURE__*/ isOnObject('catch', 'then');
