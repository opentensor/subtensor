"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.formatAbiParameter = formatAbiParameter;
const regex_js_1 = require("../regex.js");
const tupleRegex = /^tuple(?<array>(\[(\d*)\])*)$/;
function formatAbiParameter(abiParameter) {
    let type = abiParameter.type;
    if (tupleRegex.test(abiParameter.type) && 'components' in abiParameter) {
        type = '(';
        const length = abiParameter.components.length;
        for (let i = 0; i < length; i++) {
            const component = abiParameter.components[i];
            type += formatAbiParameter(component);
            if (i < length - 1)
                type += ', ';
        }
        const result = (0, regex_js_1.execTyped)(tupleRegex, abiParameter.type);
        type += `)${result?.array ?? ''}`;
        return formatAbiParameter({
            ...abiParameter,
            type,
        });
    }
    if ('indexed' in abiParameter && abiParameter.indexed)
        type = `${type} indexed`;
    if (abiParameter.name)
        return `${type} ${abiParameter.name}`;
    return type;
}
//# sourceMappingURL=formatAbiParameter.js.map