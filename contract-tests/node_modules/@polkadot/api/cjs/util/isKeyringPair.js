"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isKeyringPair = isKeyringPair;
const util_1 = require("@polkadot/util");
function isKeyringPair(account) {
    return (0, util_1.isFunction)(account.sign);
}
