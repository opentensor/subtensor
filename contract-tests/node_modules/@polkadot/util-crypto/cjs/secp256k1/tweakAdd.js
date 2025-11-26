"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.secp256k1PrivateKeyTweakAdd = secp256k1PrivateKeyTweakAdd;
const util_1 = require("@polkadot/util");
const x_bigint_1 = require("@polkadot/x-bigint");
const bn_js_1 = require("../bn.js");
const N = 'ffffffff ffffffff ffffffff fffffffe baaedce6 af48a03b bfd25e8c d0364141'.replace(/ /g, '');
const N_BI = (0, x_bigint_1.BigInt)(`0x${N}`);
const N_BN = new util_1.BN(N, 'hex');
function addBi(seckey, tweak) {
    let res = (0, util_1.u8aToBigInt)(tweak, bn_js_1.BN_BE_OPTS);
    if (res >= N_BI) {
        throw new Error('Tweak parameter is out of range');
    }
    res += (0, util_1.u8aToBigInt)(seckey, bn_js_1.BN_BE_OPTS);
    if (res >= N_BI) {
        res -= N_BI;
    }
    if (res === util_1._0n) {
        throw new Error('Invalid resulting private key');
    }
    return (0, util_1.nToU8a)(res, bn_js_1.BN_BE_256_OPTS);
}
function addBn(seckey, tweak) {
    const res = new util_1.BN(tweak);
    if (res.cmp(N_BN) >= 0) {
        throw new Error('Tweak parameter is out of range');
    }
    res.iadd(new util_1.BN(seckey));
    if (res.cmp(N_BN) >= 0) {
        res.isub(N_BN);
    }
    if (res.isZero()) {
        throw new Error('Invalid resulting private key');
    }
    return (0, util_1.bnToU8a)(res, bn_js_1.BN_BE_256_OPTS);
}
function secp256k1PrivateKeyTweakAdd(seckey, tweak, onlyBn) {
    if (!(0, util_1.isU8a)(seckey) || seckey.length !== 32) {
        throw new Error('Expected seckey to be an Uint8Array with length 32');
    }
    else if (!(0, util_1.isU8a)(tweak) || tweak.length !== 32) {
        throw new Error('Expected tweak to be an Uint8Array with length 32');
    }
    return !util_1.hasBigInt || onlyBn
        ? addBn(seckey, tweak)
        : addBi(seckey, tweak);
}
