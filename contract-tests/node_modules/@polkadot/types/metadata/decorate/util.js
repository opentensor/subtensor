import { stringCamelCase } from '@polkadot/util';
function convert(fn) {
    return ({ name }) => fn(name);
}
export const objectNameToCamel = /*#__PURE__*/ convert(stringCamelCase);
export const objectNameToString = /*#__PURE__*/ convert((n) => n.toString());
