"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.formatAbi = formatAbi;
const formatAbiItem_js_1 = require("./formatAbiItem.js");
function formatAbi(abi) {
    const signatures = [];
    const length = abi.length;
    for (let i = 0; i < length; i++) {
        const abiItem = abi[i];
        const signature = (0, formatAbiItem_js_1.formatAbiItem)(abiItem);
        signatures.push(signature);
    }
    return signatures;
}
//# sourceMappingURL=formatAbi.js.map