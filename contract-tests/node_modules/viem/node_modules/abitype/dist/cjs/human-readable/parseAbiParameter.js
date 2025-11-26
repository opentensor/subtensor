"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.parseAbiParameter = parseAbiParameter;
const abiParameter_js_1 = require("./errors/abiParameter.js");
const signatures_js_1 = require("./runtime/signatures.js");
const structs_js_1 = require("./runtime/structs.js");
const utils_js_1 = require("./runtime/utils.js");
function parseAbiParameter(param) {
    let abiParameter;
    if (typeof param === 'string')
        abiParameter = (0, utils_js_1.parseAbiParameter)(param, {
            modifiers: signatures_js_1.modifiers,
        });
    else {
        const structs = (0, structs_js_1.parseStructs)(param);
        const length = param.length;
        for (let i = 0; i < length; i++) {
            const signature = param[i];
            if ((0, signatures_js_1.isStructSignature)(signature))
                continue;
            abiParameter = (0, utils_js_1.parseAbiParameter)(signature, { modifiers: signatures_js_1.modifiers, structs });
            break;
        }
    }
    if (!abiParameter)
        throw new abiParameter_js_1.InvalidAbiParameterError({ param });
    return abiParameter;
}
//# sourceMappingURL=parseAbiParameter.js.map