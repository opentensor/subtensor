"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.createDeriveFn = createDeriveFn;
const util_1 = require("@polkadot/util");
const fromU8a_js_1 = require("./pair/fromU8a.js");
const toU8a_js_1 = require("./pair/toU8a.js");
function createDeriveFn(derive) {
    return (keypair, chainCode) => {
        if (!(0, util_1.isU8a)(chainCode) || chainCode.length !== 32) {
            throw new Error('Invalid chainCode passed to derive');
        }
        return (0, fromU8a_js_1.sr25519PairFromU8a)(derive((0, toU8a_js_1.sr25519KeypairToU8a)(keypair), chainCode));
    };
}
