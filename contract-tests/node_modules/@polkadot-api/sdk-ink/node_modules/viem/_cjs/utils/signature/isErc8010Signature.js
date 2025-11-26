"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isErc8010Signature = isErc8010Signature;
const erc8010_1 = require("ox/erc8010");
function isErc8010Signature(signature) {
    return erc8010_1.SignatureErc8010.validate(signature);
}
//# sourceMappingURL=isErc8010Signature.js.map