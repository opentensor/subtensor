"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.isTestChain = isTestChain;
const REGEX_DEV = /(Development|Local Testnet)$/;
function isTestChain(chain) {
    if (!chain) {
        return false;
    }
    return !!REGEX_DEV.test(chain.toString());
}
