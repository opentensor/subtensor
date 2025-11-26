"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.noble = void 0;
exports.createKeyPair = createKeyPair;
exports.getPublicKey = getPublicKey;
exports.randomPrivateKey = randomPrivateKey;
exports.sign = sign;
exports.verify = verify;
const ed25519_1 = require("@noble/curves/ed25519");
const Bytes = require("./Bytes.js");
const Hex = require("./Hex.js");
exports.noble = ed25519_1.ed25519;
function createKeyPair(options = {}) {
    const { as = 'Hex' } = options;
    const privateKey = randomPrivateKey({ as });
    const publicKey = getPublicKey({ privateKey, as });
    return {
        privateKey: privateKey,
        publicKey: publicKey,
    };
}
function getPublicKey(options) {
    const { as = 'Hex', privateKey } = options;
    const privateKeyBytes = Bytes.from(privateKey);
    const publicKeyBytes = ed25519_1.ed25519.getPublicKey(privateKeyBytes);
    if (as === 'Hex')
        return Hex.fromBytes(publicKeyBytes);
    return publicKeyBytes;
}
function randomPrivateKey(options = {}) {
    const { as = 'Hex' } = options;
    const bytes = ed25519_1.ed25519.utils.randomPrivateKey();
    if (as === 'Hex')
        return Hex.fromBytes(bytes);
    return bytes;
}
function sign(options) {
    const { as = 'Hex', payload, privateKey } = options;
    const payloadBytes = Bytes.from(payload);
    const privateKeyBytes = Bytes.from(privateKey);
    const signatureBytes = ed25519_1.ed25519.sign(payloadBytes, privateKeyBytes);
    if (as === 'Hex')
        return Hex.fromBytes(signatureBytes);
    return signatureBytes;
}
function verify(options) {
    const { payload, publicKey, signature } = options;
    const payloadBytes = Bytes.from(payload);
    const publicKeyBytes = Bytes.from(publicKey);
    const signatureBytes = Bytes.from(signature);
    return ed25519_1.ed25519.verify(signatureBytes, payloadBytes, publicKeyBytes);
}
//# sourceMappingURL=Ed25519.js.map