"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.format = format;
exports.from = from;
const abitype = require("abitype");
const internal = require("./internal/abi.js");
function format(abi) {
    return abitype.formatAbi(abi);
}
function from(abi) {
    if (internal.isSignatures(abi))
        return abitype.parseAbi(abi);
    return abi;
}
//# sourceMappingURL=Abi.js.map