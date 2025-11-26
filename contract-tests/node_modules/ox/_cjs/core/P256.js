"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.noble = void 0;
exports.getPublicKey = getPublicKey;
exports.randomPrivateKey = randomPrivateKey;
exports.recoverPublicKey = recoverPublicKey;
exports.sign = sign;
exports.verify = verify;
const p256_1 = require("@noble/curves/p256");
const Bytes = require("./Bytes.js");
const Hex = require("./Hex.js");
const PublicKey = require("./PublicKey.js");
const Entropy = require("./internal/entropy.js");
exports.noble = p256_1.secp256r1;
function getPublicKey(options) {
    const { privateKey } = options;
    const point = p256_1.secp256r1.ProjectivePoint.fromPrivateKey(typeof privateKey === 'string'
        ? privateKey.slice(2)
        : Hex.fromBytes(privateKey).slice(2));
    return PublicKey.from(point);
}
function randomPrivateKey(options = {}) {
    const { as = 'Hex' } = options;
    const bytes = p256_1.secp256r1.utils.randomPrivateKey();
    if (as === 'Hex')
        return Hex.fromBytes(bytes);
    return bytes;
}
function recoverPublicKey(options) {
    const { payload, signature } = options;
    const { r, s, yParity } = signature;
    const signature_ = new p256_1.secp256r1.Signature(BigInt(r), BigInt(s)).addRecoveryBit(yParity);
    const payload_ = payload instanceof Uint8Array ? Hex.fromBytes(payload) : payload;
    const point = signature_.recoverPublicKey(payload_.substring(2));
    return PublicKey.from(point);
}
function sign(options) {
    const { extraEntropy = Entropy.extraEntropy, hash, payload, privateKey, } = options;
    const { r, s, recovery } = p256_1.secp256r1.sign(payload instanceof Uint8Array ? payload : Bytes.fromHex(payload), privateKey instanceof Uint8Array ? privateKey : Bytes.fromHex(privateKey), {
        extraEntropy: typeof extraEntropy === 'boolean'
            ? extraEntropy
            : Hex.from(extraEntropy).slice(2),
        lowS: true,
        ...(hash ? { prehash: true } : {}),
    });
    return {
        r,
        s,
        yParity: recovery,
    };
}
function verify(options) {
    const { hash, payload, publicKey, signature } = options;
    return p256_1.secp256r1.verify(signature, payload instanceof Uint8Array ? payload : Bytes.fromHex(payload), PublicKey.toHex(publicKey).substring(2), ...(hash ? [{ prehash: true, lowS: true }] : []));
}
//# sourceMappingURL=P256.js.map