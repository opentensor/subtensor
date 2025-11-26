"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.parseStructs = parseStructs;
const regex_js_1 = require("../../regex.js");
const abiItem_js_1 = require("../errors/abiItem.js");
const abiParameter_js_1 = require("../errors/abiParameter.js");
const signature_js_1 = require("../errors/signature.js");
const struct_js_1 = require("../errors/struct.js");
const signatures_js_1 = require("./signatures.js");
const utils_js_1 = require("./utils.js");
function parseStructs(signatures) {
    const shallowStructs = {};
    const signaturesLength = signatures.length;
    for (let i = 0; i < signaturesLength; i++) {
        const signature = signatures[i];
        if (!(0, signatures_js_1.isStructSignature)(signature))
            continue;
        const match = (0, signatures_js_1.execStructSignature)(signature);
        if (!match)
            throw new signature_js_1.InvalidSignatureError({ signature, type: 'struct' });
        const properties = match.properties.split(';');
        const components = [];
        const propertiesLength = properties.length;
        for (let k = 0; k < propertiesLength; k++) {
            const property = properties[k];
            const trimmed = property.trim();
            if (!trimmed)
                continue;
            const abiParameter = (0, utils_js_1.parseAbiParameter)(trimmed, {
                type: 'struct',
            });
            components.push(abiParameter);
        }
        if (!components.length)
            throw new signature_js_1.InvalidStructSignatureError({ signature });
        shallowStructs[match.name] = components;
    }
    const resolvedStructs = {};
    const entries = Object.entries(shallowStructs);
    const entriesLength = entries.length;
    for (let i = 0; i < entriesLength; i++) {
        const [name, parameters] = entries[i];
        resolvedStructs[name] = resolveStructs(parameters, shallowStructs);
    }
    return resolvedStructs;
}
const typeWithoutTupleRegex = /^(?<type>[a-zA-Z$_][a-zA-Z0-9$_]*)(?<array>(?:\[\d*?\])+?)?$/;
function resolveStructs(abiParameters, structs, ancestors = new Set()) {
    const components = [];
    const length = abiParameters.length;
    for (let i = 0; i < length; i++) {
        const abiParameter = abiParameters[i];
        const isTuple = regex_js_1.isTupleRegex.test(abiParameter.type);
        if (isTuple)
            components.push(abiParameter);
        else {
            const match = (0, regex_js_1.execTyped)(typeWithoutTupleRegex, abiParameter.type);
            if (!match?.type)
                throw new abiParameter_js_1.InvalidAbiTypeParameterError({ abiParameter });
            const { array, type } = match;
            if (type in structs) {
                if (ancestors.has(type))
                    throw new struct_js_1.CircularReferenceError({ type });
                components.push({
                    ...abiParameter,
                    type: `tuple${array ?? ''}`,
                    components: resolveStructs(structs[type] ?? [], structs, new Set([...ancestors, type])),
                });
            }
            else {
                if ((0, utils_js_1.isSolidityType)(type))
                    components.push(abiParameter);
                else
                    throw new abiItem_js_1.UnknownTypeError({ type });
            }
        }
    }
    return components;
}
//# sourceMappingURL=structs.js.map