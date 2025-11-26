import { isOnObject } from './helpers.js';
const checkCodec = /*#__PURE__*/ isOnObject('toHex', 'toHuman', 'toU8a');
const checkRegistry = /*#__PURE__*/ isOnObject('get');
export function isCodec(value) {
    return checkCodec(value) && checkRegistry(value.registry);
}
