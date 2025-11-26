"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.objectNameToString = exports.objectNameToCamel = void 0;
const util_1 = require("@polkadot/util");
function convert(fn) {
    return ({ name }) => fn(name);
}
exports.objectNameToCamel = convert(util_1.stringCamelCase);
exports.objectNameToString = convert((n) => n.toString());
