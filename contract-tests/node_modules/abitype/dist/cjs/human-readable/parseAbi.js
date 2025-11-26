"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.parseAbi = parseAbi;
const signatures_js_1 = require("./runtime/signatures.js");
const structs_js_1 = require("./runtime/structs.js");
const utils_js_1 = require("./runtime/utils.js");
function parseAbi(signatures) {
    const structs = (0, structs_js_1.parseStructs)(signatures);
    const abi = [];
    const length = signatures.length;
    for (let i = 0; i < length; i++) {
        const signature = signatures[i];
        if ((0, signatures_js_1.isStructSignature)(signature))
            continue;
        abi.push((0, utils_js_1.parseSignature)(signature, structs));
    }
    return abi;
}
//# sourceMappingURL=parseAbi.js.map