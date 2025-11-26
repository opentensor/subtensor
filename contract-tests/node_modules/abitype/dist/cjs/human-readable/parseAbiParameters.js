"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.parseAbiParameters = parseAbiParameters;
const abiParameter_js_1 = require("./errors/abiParameter.js");
const signatures_js_1 = require("./runtime/signatures.js");
const structs_js_1 = require("./runtime/structs.js");
const utils_js_1 = require("./runtime/utils.js");
const utils_js_2 = require("./runtime/utils.js");
function parseAbiParameters(params) {
    const abiParameters = [];
    if (typeof params === 'string') {
        const parameters = (0, utils_js_1.splitParameters)(params);
        const length = parameters.length;
        for (let i = 0; i < length; i++) {
            abiParameters.push((0, utils_js_2.parseAbiParameter)(parameters[i], { modifiers: signatures_js_1.modifiers }));
        }
    }
    else {
        const structs = (0, structs_js_1.parseStructs)(params);
        const length = params.length;
        for (let i = 0; i < length; i++) {
            const signature = params[i];
            if ((0, signatures_js_1.isStructSignature)(signature))
                continue;
            const parameters = (0, utils_js_1.splitParameters)(signature);
            const length = parameters.length;
            for (let k = 0; k < length; k++) {
                abiParameters.push((0, utils_js_2.parseAbiParameter)(parameters[k], { modifiers: signatures_js_1.modifiers, structs }));
            }
        }
    }
    if (abiParameters.length === 0)
        throw new abiParameter_js_1.InvalidAbiParametersError({ params });
    return abiParameters;
}
//# sourceMappingURL=parseAbiParameters.js.map