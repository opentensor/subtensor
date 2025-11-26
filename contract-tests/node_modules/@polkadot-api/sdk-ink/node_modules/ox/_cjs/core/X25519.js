"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.noble = void 0;
exports.createKeyPair = createKeyPair;
exports.getPublicKey = getPublicKey;
exports.getSharedSecret = getSharedSecret;
exports.randomPrivateKey = randomPrivateKey;
const ed25519_1 = require("@noble/curves/ed25519");
const Bytes = require("./Bytes.js");
const Hex = require("./Hex.js");
exports.noble = ed25519_1.x25519;
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
    const publicKeyBytes = ed25519_1.x25519.getPublicKey(privateKeyBytes);
    if (as === 'Hex')
        return Hex.fromBytes(publicKeyBytes);
    return publicKeyBytes;
}
function getSharedSecret(options) {
    const { as = 'Hex', privateKey, publicKey } = options;
    const privateKeyBytes = Bytes.from(privateKey);
    const publicKeyBytes = Bytes.from(publicKey);
    const sharedSecretBytes = ed25519_1.x25519.getSharedSecret(privateKeyBytes, publicKeyBytes);
    if (as === 'Hex')
        return Hex.fromBytes(sharedSecretBytes);
    return sharedSecretBytes;
}
function randomPrivateKey(options = {}) {
    const { as = 'Hex' } = options;
    const bytes = ed25519_1.x25519.utils.randomPrivateKey();
    if (as === 'Hex')
        return Hex.fromBytes(bytes);
    return bytes;
}
//# sourceMappingURL=X25519.js.map